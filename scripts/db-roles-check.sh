#!/usr/bin/env bash
# scripts/db-roles-check.sh: menjalankan bootstrap klaster db/roles/*.sql DUA KALI (idempoten)
# lalu test katalog dan test negatif db/roles/test/*.sql ke PostgreSQL 18 sungguhan.
#
# Modul kritis T1 (ADR-008, ADR-021 butir 4). Dasar: docs/05-physical-schema.md §3.5, §11.1, §12.
#
# Pakai:
#   bash scripts/db-roles-check.sh ephemeral [rinci]
#       Kontainer PostgreSQL sekali pakai (image dipin sama dengan infra/local/compose.yaml),
#       tanpa jaringan dan tanpa port terbuka. Ketiga bootstrap dijalankan di SATU klaster
#       (nama role berawalan, §3.1 butir 2), sehingga isolasi lintas sistem ikut teruji.
#       Dipakai scripts/verify.sh.
#   bash scripts/db-roles-check.sh compose [rinci]
#       Klaster compose lokal yang sedang berjalan (infra/local/compose.yaml): app.sql ke
#       postgres-app, audit.sql ke postgres-audit, rme.sql ke postgres-rme (tata letak production).
#       Ini juga cara menerapkan bootstrap di lingkungan lokal.
#   "rinci" mencetak setiap baris LULUS; tanpa itu hanya jumlah per tahap (galat selalu dicetak).
#
# Test berpindah role lewat soket lokal di dalam kontainer (trust pada image resmi postgres),
# sehingga tidak ada password yang dibuat, disimpan, atau dikirim. Jangan dipakai ke production.
# Portabel bash 3.2 (Linux dan macOS), tanpa array asosiatif.

set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "pakai: bash scripts/db-roles-check.sh ephemeral|compose [rinci]" >&2
  exit 2
fi
MODE="$1"
RINCI="${2:-}"
case "$MODE" in ephemeral | compose) ;; *)
  echo "mode tidak dikenal: $MODE" >&2
  exit 2
  ;;
esac
if [ -n "$RINCI" ] && [ "$RINCI" != "rinci" ]; then
  echo "argumen kedua hanya boleh 'rinci'" >&2
  exit 2
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

LULUS=0
GAGAL=0
CONTAINER=""
# Semua berkas sementara di satu direktori kerja yang selalu dihapus saat keluar (review F1-T13).
KERJA="$(mktemp -d)"

bersihkan() {
  if [ -n "$CONTAINER" ]; then
    docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
  fi
  rm -rf "$KERJA"
}
trap bersihkan EXIT

if ! command -v docker >/dev/null 2>&1; then
  echo "docker tidak ditemukan di PATH (dibutuhkan untuk PostgreSQL uji)" >&2
  exit 1
fi

for f in db/roles/app.sql db/roles/audit.sql db/roles/rme.sql \
  db/roles/test/cluster.sql db/roles/test/app.sql db/roles/test/audit.sql db/roles/test/rme.sql \
  db/roles/test/umpan-tanam.sql db/roles/test/umpan-periksa.sql; do
  if [ ! -f "$f" ]; then
    echo "berkas tidak ada: $f" >&2
    exit 1
  fi
done

tandai() { # tandai <lulus|gagal> <pesan>
  if [ "$1" = "lulus" ]; then
    LULUS=$((LULUS + 1))
    [ -n "$RINCI" ] && echo "  LULUS $2"
  else
    GAGAL=$((GAGAL + 1))
    echo "  GAGAL $2" >&2
  fi
  return 0
}

# psql superuser di dalam kontainer: psql_su <kontainer> <database> [opsi psql...] (SQL dari stdin)
psql_su() {
  local ctr="$1" db="$2"
  shift 2
  docker exec -i -u postgres "$ctr" psql -X -q -v ON_ERROR_STOP=1 -U postgres -d "$db" "$@"
}

# --- Pemeriksaan statis: bootstrap tanpa password (§3.5, coding-standards §5.3) ---
statis() {
  local temuan
  temuan="$(grep -n -i 'password' db/roles/*.sql | grep -v -E '^[^:]+:[0-9]+:[[:space:]]*--' || true)"
  if [ -n "$temuan" ]; then
    tandai gagal "statis: db/roles/*.sql memuat PASSWORD di luar komentar: $temuan"
  else
    tandai lulus "statis: db/roles/*.sql tanpa password"
  fi
}

# --- Potret katalog untuk membandingkan hasil bootstrap pertama dan kedua ---
potret() { # potret <kontainer>
  local ctr="$1" dbs db
  psql_su "$ctr" postgres -At <<'SQL'
SET search_path = pg_catalog, pg_temp;
SELECT 'role', rolname, rolsuper, rolinherit, rolcreaterole, rolcreatedb, rolcanlogin, rolreplication,
       rolbypassrls, rolconnlimit
  FROM pg_catalog.pg_roles WHERE rolname !~ '^pg_' ORDER BY rolname;
SELECT 'member', g.rolname, m.rolname, a.admin_option, a.inherit_option, a.set_option
  FROM pg_catalog.pg_auth_members a
  JOIN pg_catalog.pg_roles g ON g.oid = a.roleid JOIN pg_catalog.pg_roles m ON m.oid = a.member
 ORDER BY 2, 3;
SELECT 'database', datname, pg_catalog.pg_get_userbyid(datdba), datacl FROM pg_catalog.pg_database ORDER BY datname;
SELECT 'setting', coalesce(d.datname, '-'), coalesce(r.rolname, '-'), s.setconfig
  FROM pg_catalog.pg_db_role_setting s
  LEFT JOIN pg_catalog.pg_database d ON d.oid = s.setdatabase
  LEFT JOIN pg_catalog.pg_roles r ON r.oid = s.setrole
 ORDER BY 2, 3;
SQL
  dbs="$(psql_su "$ctr" postgres -At -c "SELECT datname FROM pg_catalog.pg_database WHERE datname IN ('ruang_sehat','ruang_sehat_audit','rme_audit','rme') ORDER BY datname")"
  for db in $dbs; do
    psql_su "$ctr" "$db" -At <<SQL
SET search_path = pg_catalog, pg_temp;
SELECT '$db', 'schema', nspname, pg_catalog.pg_get_userbyid(nspowner), nspacl FROM pg_catalog.pg_namespace ORDER BY nspname;
SELECT '$db', 'extension', extname, extnamespace::regnamespace, extversion FROM pg_catalog.pg_extension ORDER BY extname;
SELECT '$db', 'default_acl', defaclrole::regrole, defaclnamespace, defaclobjtype, defaclacl
  FROM pg_catalog.pg_default_acl ORDER BY 3, 4, 5;
SQL
  done
}

# bootstrap_dua_kali <label> <pasangan "kontainer:berkas" ...>
bootstrap_dua_kali() {
  local label="$1" p ctr file putaran out tmp1="$KERJA/potret-1" tmp2="$KERJA/potret-2" tujuan
  shift
  : >"$tmp1"
  : >"$tmp2"
  for putaran in 1 2; do
    for p in "$@"; do
      ctr="${p%%:*}"
      file="${p#*:}"
      if out="$(psql_su "$ctr" postgres <"$file" 2>&1)"; then
        tandai lulus "bootstrap putaran $putaran: $file ($label)"
      else
        tandai gagal "bootstrap putaran $putaran: $file ($label): $out"
        return 0
      fi
    done
    if [ "$putaran" = 1 ]; then tujuan="$tmp1"; else tujuan="$tmp2"; fi
    for p in "$@"; do
      potret "${p%%:*}" >>"$tujuan"
    done
  done
  if diff -u "$tmp1" "$tmp2" >/dev/null; then
    tandai lulus "idempoten: katalog setelah putaran 1 dan 2 identik ($label, $(wc -l <"$tmp1" | tr -d ' ') baris potret)"
  else
    tandai gagal "idempoten: katalog berubah pada putaran 2 ($label): $(diff -u "$tmp1" "$tmp2" || true)"
  fi
}

# uji_umpan <label> <"kontainer:database:skema:pemilik:sasaran" ...> -- <"kontainer:berkas" ...>
# Test negatif pembajakan search_path (pola CVE-2018-1058; review keamanan F1-T13): fungsi umpan
# ditanam pemilik skema, bootstrap dijalankan ulang, lalu dipastikan tidak ada umpan terpanggil,
# role sasaran tetap bukan superuser, dan potret katalog sama dengan sebelum umpan ditanam.
uji_umpan() {
  local label="$1" t p ctr db sk pm ss out n umpan="" boot="" sebelum="$KERJA/umpan-1" sesudah="$KERJA/umpan-2"
  shift
  while [ "$#" -gt 0 ] && [ "$1" != "--" ]; do
    umpan="$umpan $1"
    shift
  done
  shift
  boot="$*"
  : >"$sebelum"
  : >"$sesudah"
  for p in $boot; do potret "${p%%:*}" >>"$sebelum"; done
  for t in $umpan; do
    IFS=: read -r ctr db sk pm ss <<UMPAN
$t
UMPAN
    if ! out="$(psql_su "$ctr" "$db" -v skema="$sk" -v pemilik="$pm" -v sasaran="$ss" <db/roles/test/umpan-tanam.sql 2>&1)"; then
      tandai gagal "NEG pembajakan search_path: gagal menanam umpan di $db.$sk: $out"
    fi
  done
  for p in $boot; do
    if out="$(psql_su "${p%%:*}" postgres <"${p#*:}" 2>&1)"; then
      tandai lulus "bootstrap ulang di atas umpan: ${p#*:} ($label)"
    else
      tandai gagal "bootstrap ulang di atas umpan: ${p#*:} ($label): $out"
    fi
  done
  for p in $boot; do potret "${p%%:*}" >>"$sesudah"; done
  # Pemeriksaan sekaligus pembersihan umpan; selalu dijalankan untuk setiap lokasi.
  for t in $umpan; do
    IFS=: read -r ctr db sk pm ss <<UMPAN
$t
UMPAN
    if out="$(psql_su "$ctr" "$db" -v skema="$sk" -v sasaran="$ss" <db/roles/test/umpan-periksa.sql 2>&1)"; then
      n="$(printf '%s\n' "$out" | grep -c 'NOTICE:  LULUS ' || true)"
      if [ "$n" -eq 1 ]; then
        tandai lulus "${out##*NOTICE:  LULUS }"
      else
        tandai gagal "NEG pembajakan search_path $db.$sk: keluaran tak terduga: $out"
      fi
    else
      tandai gagal "$(printf '%s\n' "$out" | grep -E 'ERROR|GAGAL' || printf '%s' "$out")"
    fi
  done
  if diff -u "$sebelum" "$sesudah" >/dev/null; then
    tandai lulus "NEG pembajakan search_path: potret katalog (role, keanggotaan, ACL, pengaturan) tidak berubah ($label)"
  else
    tandai gagal "NEG pembajakan search_path: katalog berubah ($label): $(diff -u "$sebelum" "$sesudah" || true)"
  fi
}

# jalankan_test <kontainer> <berkas test>: setiap "LULUS ..." dihitung; galat menghentikan berkas.
jalankan_test() {
  local ctr="$1" file="$2" out n baris
  if out="$(psql_su "$ctr" postgres <"$file" 2>&1)"; then
    n=0
    while IFS= read -r baris; do
      case "$baris" in
        *"NOTICE:  LULUS "*)
          n=$((n + 1))
          tandai lulus "${baris#*NOTICE:  LULUS }"
          ;;
      esac
    done <<EOF
$out
EOF
    [ -z "$RINCI" ] && echo "  $file: $n pemeriksaan lulus"
    if [ "$n" -eq 0 ]; then
      tandai gagal "$file: tidak ada pemeriksaan yang berjalan"
    fi
  else
    tandai gagal "$file: $(printf '%s\n' "$out" | grep -E 'ERROR|GAGAL|FATAL' || printf '%s' "$out")"
  fi
}

# tolak_koneksi <kontainer> <role> <database> <pola galat>: login sungguhan wajib ditolak.
tolak_koneksi() {
  local ctr="$1" role="$2" db="$3" pola="$4" out
  if out="$(docker exec -u postgres "$ctr" psql -X -w -At -U "$role" -d "$db" -c 'SELECT 1' 2>&1)"; then
    tandai gagal "NEG koneksi $role -> $db seharusnya ditolak, tetapi berhasil"
  elif printf '%s' "$out" | grep -q -- "$pola"; then
    tandai lulus "NEG koneksi $role -> $db ditolak ($pola)"
  else
    tandai gagal "NEG koneksi $role -> $db ditolak dengan galat tak terduga: $out"
  fi
}

DB_DITOLAK="permission denied for database"
TANPA_LOGIN="is not permitted to log in"

koneksi_klaster_a() {
  local c="$1"
  tolak_koneksi "$c" rs_api keycloak "$DB_DITOLAK"
  tolak_koneksi "$c" rs_api postgres "$DB_DITOLAK"
  tolak_koneksi "$c" rs_metrics keycloak "$DB_DITOLAK"
  tolak_koneksi "$c" openbao_rs_admin keycloak "$DB_DITOLAK"
  tolak_koneksi "$c" keycloak ruang_sehat "$DB_DITOLAK"
  tolak_koneksi "$c" keycloak postgres "$DB_DITOLAK"
  tolak_koneksi "$c" rs_app ruang_sehat "$TANPA_LOGIN"
  tolak_koneksi "$c" rs_migration ruang_sehat "$TANPA_LOGIN"
}

koneksi_klaster_b() {
  local c="$1"
  tolak_koneksi "$c" rs_audit_writer_login rme_audit "$DB_DITOLAK"
  tolak_koneksi "$c" rs_audit_reader_login rme_audit "$DB_DITOLAK"
  tolak_koneksi "$c" openbao_rs_audit_admin rme_audit "$DB_DITOLAK"
  tolak_koneksi "$c" rme_audit_tenant_reader_login ruang_sehat_audit "$DB_DITOLAK"
  tolak_koneksi "$c" rme_audit_writer_login ruang_sehat_audit "$DB_DITOLAK"
  tolak_koneksi "$c" openbao_rme_audit_admin ruang_sehat_audit "$DB_DITOLAK"
  tolak_koneksi "$c" rme_audit_verifier_login postgres "$DB_DITOLAK"
  tolak_koneksi "$c" rs_audit_writer ruang_sehat_audit "$TANPA_LOGIN"
  tolak_koneksi "$c" rme_audit_migration rme_audit "$TANPA_LOGIN"
}

koneksi_klaster_c() {
  local c="$1"
  tolak_koneksi "$c" rme_api postgres "$DB_DITOLAK"
  tolak_koneksi "$c" rme_api_auth postgres "$DB_DITOLAK"
  tolak_koneksi "$c" openbao_rme_admin postgres "$DB_DITOLAK"
  tolak_koneksi "$c" rme_app rme "$TANPA_LOGIN"
  tolak_koneksi "$c" rme_migration rme "$TANPA_LOGIN"
}

# Hanya bermakna bila ketiga bootstrap berada di satu klaster (mode ephemeral).
koneksi_lintas_sistem() {
  local c="$1"
  tolak_koneksi "$c" rs_api rme "$DB_DITOLAK"
  tolak_koneksi "$c" rs_api ruang_sehat_audit "$DB_DITOLAK"
  tolak_koneksi "$c" rs_audit_relay_login ruang_sehat_audit "$DB_DITOLAK"
  tolak_koneksi "$c" rme_api ruang_sehat "$DB_DITOLAK"
  tolak_koneksi "$c" rme_api keycloak "$DB_DITOLAK"
  tolak_koneksi "$c" rme_audit_tenant_reader_login rme "$DB_DITOLAK"
  tolak_koneksi "$c" keycloak rme_audit "$DB_DITOLAK"
}

tunggu_siap() { # tunggu_siap <kontainer>: TCP lokal hanya hidup setelah inisialisasi image selesai
  local i
  for i in $(seq 1 90); do
    if docker exec "$1" pg_isready -q -h 127.0.0.1 -U postgres >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  echo "PostgreSQL di kontainer $1 tidak siap dalam 90 detik" >&2
  return 1
}

echo "db-roles-check: mode $MODE"
statis

if [ "$MODE" = "ephemeral" ]; then
  IMAGE="$(sed -n 's/^[[:space:]]*image:[[:space:]]*\(postgres:[^[:space:]]*@sha256:[0-9a-f]*\)[[:space:]]*$/\1/p' infra/local/compose.yaml | sed -n '1p')"
  if [ -z "$IMAGE" ]; then
    echo "image postgres (tag@digest) tidak terbaca dari infra/local/compose.yaml" >&2
    exit 1
  fi
  CONTAINER="rs-db-roles-check-$$-$RANDOM"
  # Password superuser acak sekali pakai hanya karena image mewajibkannya; tidak dipakai (soket trust).
  docker run -d --rm --network none --name "$CONTAINER" \
    -e POSTGRES_PASSWORD="sekali-pakai-$RANDOM$RANDOM$RANDOM" "$IMAGE" >/dev/null
  tunggu_siap "$CONTAINER"
  echo "  kontainer $CONTAINER ($IMAGE), satu klaster untuk app + audit + rme"
  bootstrap_dua_kali "satu klaster" \
    "$CONTAINER:db/roles/app.sql" "$CONTAINER:db/roles/audit.sql" "$CONTAINER:db/roles/rme.sql"
  if [ "$GAGAL" -eq 0 ]; then
    jalankan_test "$CONTAINER" db/roles/test/cluster.sql
    jalankan_test "$CONTAINER" db/roles/test/app.sql
    jalankan_test "$CONTAINER" db/roles/test/audit.sql
    jalankan_test "$CONTAINER" db/roles/test/rme.sql
    koneksi_klaster_a "$CONTAINER"
    koneksi_klaster_b "$CONTAINER"
    koneksi_klaster_c "$CONTAINER"
    koneksi_lintas_sistem "$CONTAINER"
    uji_umpan "satu klaster" \
      "$CONTAINER:ruang_sehat:app:rs_migration:rs_api" \
      "$CONTAINER:ruang_sehat:util:rs_migration:rs_worker" \
      "$CONTAINER:ruang_sehat_audit:audit:rs_audit_migration:rs_audit_reader_login" \
      "$CONTAINER:ruang_sehat_audit:util:rs_audit_migration:rs_audit_writer_login" \
      "$CONTAINER:rme_audit:audit:rme_audit_migration:rme_audit_tenant_reader_login" \
      "$CONTAINER:rme_audit:util:rme_audit_migration:rme_audit_writer_login" \
      "$CONTAINER:rme:app:rme_migration:rme_api" \
      "$CONTAINER:rme:util:rme_migration:rme_worker_login" \
      -- "$CONTAINER:db/roles/app.sql" "$CONTAINER:db/roles/audit.sql" "$CONTAINER:db/roles/rme.sql"
  fi
else
  COMPOSE="docker compose -f infra/local/compose.yaml"
  C_APP="$($COMPOSE ps -q postgres-app)"
  C_AUDIT="$($COMPOSE ps -q postgres-audit)"
  C_RME="$($COMPOSE ps -q postgres-rme)"
  if [ -z "$C_APP" ] || [ -z "$C_AUDIT" ] || [ -z "$C_RME" ]; then
    echo "kontainer postgres-app/postgres-audit/postgres-rme belum berjalan; jalankan dulu:" >&2
    echo "  docker compose -f infra/local/compose.yaml up -d --wait" >&2
    exit 1
  fi
  for c in "$C_APP" "$C_AUDIT" "$C_RME"; do tunggu_siap "$c"; done
  echo "  klaster compose: postgres-app, postgres-audit, postgres-rme"
  bootstrap_dua_kali "tiga klaster compose" \
    "$C_APP:db/roles/app.sql" "$C_AUDIT:db/roles/audit.sql" "$C_RME:db/roles/rme.sql"
  if [ "$GAGAL" -eq 0 ]; then
    jalankan_test "$C_APP" db/roles/test/cluster.sql
    jalankan_test "$C_AUDIT" db/roles/test/cluster.sql
    jalankan_test "$C_RME" db/roles/test/cluster.sql
    jalankan_test "$C_APP" db/roles/test/app.sql
    jalankan_test "$C_AUDIT" db/roles/test/audit.sql
    jalankan_test "$C_RME" db/roles/test/rme.sql
    koneksi_klaster_a "$C_APP"
    koneksi_klaster_b "$C_AUDIT"
    koneksi_klaster_c "$C_RME"
    uji_umpan "tiga klaster compose" \
      "$C_APP:ruang_sehat:app:rs_migration:rs_api" \
      "$C_APP:ruang_sehat:util:rs_migration:rs_worker" \
      "$C_AUDIT:ruang_sehat_audit:audit:rs_audit_migration:rs_audit_reader_login" \
      "$C_AUDIT:ruang_sehat_audit:util:rs_audit_migration:rs_audit_writer_login" \
      "$C_AUDIT:rme_audit:audit:rme_audit_migration:rme_audit_tenant_reader_login" \
      "$C_AUDIT:rme_audit:util:rme_audit_migration:rme_audit_writer_login" \
      "$C_RME:rme:app:rme_migration:rme_api" \
      "$C_RME:rme:util:rme_migration:rme_worker_login" \
      -- "$C_APP:db/roles/app.sql" "$C_AUDIT:db/roles/audit.sql" "$C_RME:db/roles/rme.sql"
  fi
fi

echo "db-roles-check ($MODE): $LULUS lulus, $GAGAL gagal"
if [ "$GAGAL" -ne 0 ] || [ "$LULUS" -eq 0 ]; then
  exit 1
fi
