#!/usr/bin/env bash
# scripts/db-migrations-check.sh: langkah 2b scripts/verify.sh (F1-T15). Menerapkan semua migrasi
# db/migrations/<app|audit|rme>/ dengan sqlx-cli ke PostgreSQL 18 kosong sekali pakai, sebagai role
# migrasi dinamis tiruan OpenBao (bukan superuser), lalu test katalog wajib dan test negatif.
#
# Modul kritis T1 (ADR-008, ADR-021 butir 4). Dasar: docs/05-physical-schema.md §3.4, §3.5, §4,
# §11, §12 (T-01–T-09, T-18, T-22–T-24); KEPUTUSAN-82, KEPUTUSAN-140, KEPUTUSAN-158.
#
# Urutan:
#   1. Pemeriksaan teks berkas migrasi (nama, maju saja, penjaga di awal berkas, T-23) + test
#      negatif pemeriksa dengan berkas rusak buatan.
#   2. Kontainer PostgreSQL (image dipin sama dengan infra/local/compose.yaml dan
#      scripts/db-roles-check.sh), hanya terbuka di 127.0.0.1 port acak; bootstrap db/roles/*.sql.
#   3. Test negatif penjaga SEBELUM migrasi: setiap berkas dijalankan role salah (superuser, role
#      aplikasi, role dinamis tanpa "SET role") dan pada urutan/target salah (bootstrap belum
#      lengkap, database salah, pemilik sistem lain) wajib ditolak tanpa mengubah katalog.
#   4. sqlx migrate run per database dengan role dinamis tiruan (§3.4 creation_statements oleh
#      openbao_*_admin, password acak sekali pakai lewat TCP lokal); dijalankan DUA KALI
#      (idempoten: putaran kedua tanpa migrasi baru dan katalog identik); berkas yang diubah
#      setelah diterapkan wajib ditolak checksum sqlx.
#   5. Test katalog db/test/*.sql per database; role/keanggotaan tidak berubah oleh migrasi;
#      revocation_statements (DROP ROLE) berhasil = role dinamis tidak memiliki objek (T-04).
# Kontainer dan direktori kerja selalu dihapus (trap). Portabel bash 3.2, tanpa `head`.
# Pakai: bash scripts/db-migrations-check.sh [rinci]

set -euo pipefail

if [ "$#" -gt 1 ] || { [ "$#" -eq 1 ] && [ "$1" != "rinci" ]; }; then
  echo "pakai: bash scripts/db-migrations-check.sh [rinci]" >&2
  exit 2
fi
RINCI="${1:-}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# sqlx-cli dipin (docs/05-stack.md §1.1, SRC-182, SRC-399). Dicari dulu di lokasi khusus versi
# (tidak bentrok dengan sqlx-cli versi lain di ~/.cargo/bin milik proyek lain di mesin yang sama),
# lalu di PATH; versinya selalu diperiksa.
SQLX_VERSI="0.9.1"
SQLX_ROOT="$HOME/.local/share/ruang-sehat/sqlx-cli-$SQLX_VERSI"

LULUS=0
GAGAL=0
CONTAINER=""
KERJA="$(mktemp -d)"

bersihkan() {
  if [ -n "$CONTAINER" ]; then
    docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
  fi
  rm -rf "$KERJA"
}
trap bersihkan EXIT

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

for alat in docker awk od; do
  if ! command -v "$alat" >/dev/null 2>&1; then
    echo "$alat tidak ditemukan di PATH" >&2
    exit 1
  fi
done
SQLX="$SQLX_ROOT/bin/sqlx"
if [ ! -x "$SQLX" ]; then
  SQLX="$(command -v sqlx || true)"
fi
VERSI_AKTIF=""
[ -n "$SQLX" ] && VERSI_AKTIF="$("$SQLX" --version | sed -n '1p' | sed 's/^sqlx-cli //')"
if [ "$VERSI_AKTIF" != "$SQLX_VERSI" ]; then
  echo "sqlx-cli $SQLX_VERSI tidak ditemukan (di $SQLX_ROOT/bin maupun PATH: ${VERSI_AKTIF:-tidak ada}); docs/05-stack.md §1.1" >&2
  echo "  pasang: cargo install sqlx-cli --version $SQLX_VERSI --locked --no-default-features --features postgres,rustls --root $SQLX_ROOT" >&2
  exit 1
fi
for f in db/test/lib.sql db/test/katalog.sql db/test/util-app.sql db/test/kelas-app.sql \
  db/test/util-audit.sql db/test/kelas-audit.sql db/roles/app.sql db/roles/audit.sql db/roles/rme.sql; do
  if [ ! -f "$f" ]; then
    echo "berkas tidak ada: $f" >&2
    exit 1
  fi
done

acak() { od -An -N16 -tx1 /dev/urandom | tr -d ' \n'; }

# ---------------------------------------------------------------------------------------------
# 1. Pemeriksaan teks berkas migrasi
# ---------------------------------------------------------------------------------------------
# temuan_statis <direktori> <app|audit|rme>: mencetak satu baris per temuan (kosong = bersih).
temuan_statis() {
  local dir="$1" sistem="$2" f nama penjaga datar versi semua_versi=""
  for f in "$dir"/* "$dir"/.[!.]*; do
    [ -e "$f" ] || continue
    nama="$(basename "$f")"
    [ "$nama" = ".gitkeep" ] && continue
    if [ ! -f "$f" ]; then
      echo "$nama: bukan berkas biasa"
      continue
    fi
    # Nama: <versi 14 digit>_<deskripsi snake_case>.sql; tanpa .up/.down (maju saja, ADR-019).
    if ! printf '%s\n' "$nama" | grep -Eq '^[0-9]{14}_[a-z0-9]+(_[a-z0-9]+)*\.sql$'; then
      echo "$nama: nama bukan <versi 14 digit>_<snake_case>.sql (ADR-001; maju saja tanpa .up/.down, ADR-019)"
      continue
    fi
    versi="${nama%%_*}"
    case " $semua_versi " in *" $versi "*) echo "$nama: versi $versi ganda" ;; esac
    semua_versi="$semua_versi $versi"
    # sqlx menjalankan berkas tanpa transaksi bila diawali "-- no-transaction" (SRC-399); penjaga
    # SET LOCAL dan atomisitas migrasi bergantung pada transaksi.
    if sed -n '1p' "$f" | grep -q '^-- no-transaction'; then
      echo "$nama: diawali -- no-transaction (migrasi wajib dalam transaksi, §11.2)"
    fi
    # Penjaga (§11.2): pernyataan pertama blok DO yang memeriksa current_user terhadap pemilik,
    # diikuti SET LOCAL lock_timeout dan statement_timeout.
    penjaga="$(awk '
      s == 0 && /^[[:space:]]*(--.*)?$/ { next }
      s == 0 && /^DO \$\$[[:space:]]*$/ { s = 1; print; next }
      s == 0 { print "<<BUKAN-PENJAGA>>"; exit }
      s == 1 { print; if ($0 ~ /^END \$\$;[[:space:]]*$/) s = 2; next }
      s == 2 && /^[[:space:]]*(--.*)?$/ { next }
      s == 2 { print; n++; if (n == 2) exit }
    ' "$f")"
    if printf '%s\n' "$penjaga" | grep -q '<<BUKAN-PENJAGA>>' || ! printf '%s\n' "$penjaga" | grep -q '^END \$\$;'; then
      echo "$nama: pernyataan pertama bukan blok penjaga DO \$\$ ... END \$\$; (§11.2)"
    else
      if ! printf '%s\n' "$penjaga" | grep -q 'current_user' ||
        ! printf '%s\n' "$penjaga" | grep -q "RAISE EXCEPTION 'migration must run as"; then
        echo "$nama: penjaga tidak memeriksa current_user dengan pesan 'migration must run as' (§11.2)"
      fi
      case "$sistem" in
        app) printf '%s\n' "$penjaga" | grep -q "'rs_migration'" || echo "$nama: penjaga tidak memeriksa rs_migration" ;;
        rme) printf '%s\n' "$penjaga" | grep -q "'rme_migration'" || echo "$nama: penjaga tidak memeriksa rme_migration" ;;
        audit)
          if ! printf '%s\n' "$penjaga" | grep -q "util.fn_audit_role('audit_migration')" &&
            ! { printf '%s\n' "$penjaga" | grep -q "'rs_audit_migration'" &&
              printf '%s\n' "$penjaga" | grep -q "'rme_audit_migration'"; }; then
            echo "$nama: penjaga tidak memeriksa util.fn_audit_role('audit_migration') (§11.2)"
          fi
          ;;
      esac
      if ! printf '%s\n' "$penjaga" | grep -q "^SET LOCAL lock_timeout = '5s';" ||
        ! printf '%s\n' "$penjaga" | grep -q "^SET LOCAL statement_timeout = '15min';"; then
        echo "$nama: penjaga tidak diikuti SET LOCAL lock_timeout = '5s'; dan SET LOCAL statement_timeout = '15min'; (§11.2)"
      fi
    fi
    # T-23 (§11.4, KEPUTUSAN-158): tanpa DISABLE TRIGGER (bentuk apa pun), tanpa DROP/ALTER TRIGGER
    # atas trigger penjaga, tanpa session_replication_role. Tidak peka huruf besar-kecil, lintas baris.
    datar="$(tr '\n\t' '  ' <"$f")"
    if printf '%s\n' "$datar" | grep -Eiq 'DISABLE[[:space:]]+TRIGGER'; then
      echo "$nama: memuat DISABLE TRIGGER (T-23, §11.4)"
    fi
    if printf '%s\n' "$datar" | grep -Eiq '(DROP|ALTER)[[:space:]]+TRIGGER[[:space:]]+(IF[[:space:]]+EXISTS[[:space:]]+)?"?tg_[a-z0-9_]*_(reject_[a-z0-9_]*|guard_delete|one_way|effective_lock|no_backdate|published_lock|single_source)\b'; then
      echo "$nama: memuat DROP/ALTER TRIGGER atas trigger penjaga (T-23, §11.4)"
    fi
    if printf '%s\n' "$datar" | grep -Eiq 'session_replication_role'; then
      echo "$nama: memuat session_replication_role (mematikan trigger, T-23)"
    fi
  done
}

statis() {
  local sistem t
  for sistem in app audit rme; do
    t="$(temuan_statis "db/migrations/$sistem" "$sistem")"
    if [ -n "$t" ]; then
      tandai gagal "statis db/migrations/$sistem: $t"
    else
      tandai lulus "statis db/migrations/$sistem: nama, maju saja, penjaga di awal berkas, T-23 ($(find "db/migrations/$sistem" -maxdepth 1 -name '*.sql' | wc -l | tr -d ' ') berkas)"
    fi
  done
}

# statis_negatif: pemeriksa teks wajib menolak berkas rusak buatan.
PENJAGA_APP="DO \$\$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
END \$\$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';"
statis_negatif() {
  local no=0 d t
  kasus() { # kasus <label> <sistem> <nama berkas> <isi> <pola temuan>
    no=$((no + 1))
    d="$KERJA/statis-neg-$no"
    mkdir -p "$d"
    printf '%s\n' "$4" >"$d/$3"
    t="$(temuan_statis "$d" "$2")"
    if printf '%s\n' "$t" | grep -q -- "$5"; then
      tandai lulus "NEG statis: $1"
    else
      tandai gagal "NEG statis: $1 tidak terdeteksi (temuan: ${t:-kosong})"
    fi
  }
  kasus "berkas tanpa penjaga" app 20990101000000_tanpa_penjaga.sql "CREATE TABLE app.x (id uuid);" "bukan blok penjaga"
  kasus "penjaga memeriksa role sistem lain" app 20990101000000_role_salah.sql "${PENJAGA_APP//rs_migration/rme_migration}" "tidak memeriksa rs_migration"
  kasus "penjaga tanpa SET LOCAL batas waktu" app 20990101000000_tanpa_batas.sql "$(printf '%s\n' "$PENJAGA_APP" | sed '/SET LOCAL/d')
CREATE TABLE app.x (id uuid);" "tidak diikuti SET LOCAL"
  kasus "penjaga audit tanpa fn_audit_role" audit 20990101000000_audit.sql "${PENJAGA_APP//rs_migration/rs_audit_migration}" "fn_audit_role"
  kasus "DISABLE TRIGGER ALL" app 20990101000000_matikan.sql "$PENJAGA_APP
ALTER TABLE app.x
  disable  trigger ALL;" "DISABLE TRIGGER"
  kasus "DROP TRIGGER IF EXISTS penjaga" app 20990101000000_lepas.sql "$PENJAGA_APP
DROP TRIGGER IF EXISTS tg_journal_line_reject_ud ON app.journal_line;" "DROP/ALTER TRIGGER"
  kasus "ALTER TRIGGER penjaga (ganti nama)" app 20990101000000_ganti.sql "$PENJAGA_APP
ALTER TRIGGER tg_data_key_one_way ON app.data_key RENAME TO x;" "DROP/ALTER TRIGGER"
  kasus "session_replication_role" app 20990101000000_replika.sql "$PENJAGA_APP
SET LOCAL session_replication_role = replica;" "session_replication_role"
  kasus "berkas -- no-transaction" app 20990101000000_tanpa_tx.sql "-- no-transaction
$PENJAGA_APP" "no-transaction"
  kasus "migrasi reversible .up.sql" app 20990101000000_balik.up.sql "$PENJAGA_APP" "maju saja"
  kasus "nama bukan snake_case" app 20990101000000_BuatTabel.sql "$PENJAGA_APP" "snake_case"
  kasus "berkas bukan .sql di direktori migrasi" app catatan.txt "x" "nama bukan"
}

# ---------------------------------------------------------------------------------------------
# 2. Kontainer dan helper psql
# ---------------------------------------------------------------------------------------------
psql_su() { # psql_su <database> [opsi psql...] (SQL dari stdin), sebagai superuser lewat soket
  local db="$1"
  shift
  docker exec -i -u postgres "$CONTAINER" psql -X -q -v ON_ERROR_STOP=1 -U postgres -d "$db" "$@"
}
psql_as() { # psql_as <role> <database> [opsi psql...]: login sungguhan lewat soket lokal (trust)
  local role="$1" db="$2"
  shift 2
  docker exec -i -u postgres "$CONTAINER" psql -X -q -v ON_ERROR_STOP=1 -U "$role" -d "$db" "$@"
}

tunggu_siap() {
  local i
  for i in $(seq 1 90); do
    if docker exec "$CONTAINER" pg_isready -q -h 127.0.0.1 -U postgres >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  echo "PostgreSQL di kontainer $CONTAINER tidak siap dalam 90 detik" >&2
  return 1
}

# potret: katalog yang wajib tidak berubah oleh percobaan yang ditolak dan oleh migrasi ulang.
potret() {
  local db
  psql_su postgres -At <<'SQL'
SET search_path = pg_catalog, pg_temp;
SELECT 'role', rolname, rolsuper, rolinherit, rolcreaterole, rolcreatedb, rolcanlogin, rolreplication, rolbypassrls, rolconnlimit
  FROM pg_roles WHERE rolname !~ '^pg_' AND rolname !~ '^v-' ORDER BY rolname;
SELECT 'member', g.rolname, m.rolname, a.admin_option, a.inherit_option, a.set_option
  FROM pg_auth_members a JOIN pg_roles g ON g.oid = a.roleid JOIN pg_roles m ON m.oid = a.member
 WHERE m.rolname !~ '^v-' AND g.rolname !~ '^v-' ORDER BY 2, 3;
SELECT 'database', datname, pg_get_userbyid(datdba), datacl FROM pg_database ORDER BY datname;
SQL
  for db in ruang_sehat ruang_sehat_audit rme_audit rme; do
    psql_su "$db" -At <<SQL
SET search_path = pg_catalog, pg_temp;
SELECT '$db', 'schema', nspname, pg_get_userbyid(nspowner), nspacl FROM pg_namespace ORDER BY nspname;
SELECT '$db', 'rel', n.nspname, c.relname, c.relkind, pg_get_userbyid(c.relowner), c.relacl, obj_description(c.oid, 'pg_class')
  FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
 WHERE n.nspname IN ('app', 'util', 'audit', 'audit_detached', 'ext') ORDER BY 3, 4;
SELECT '$db', 'proc', p.oid::regprocedure, pg_get_userbyid(p.proowner), p.proacl, p.prosecdef, p.proconfig, md5(p.prosrc)
  FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
 WHERE n.nspname IN ('app', 'util', 'audit', 'audit_detached') ORDER BY 3;
SELECT '$db', 'trigger', tgrelid::regclass, tgname, tgenabled, tgfoid::regproc FROM pg_trigger WHERE NOT tgisinternal ORDER BY 3, 4;
SQL
    psql_su "$db" -At <<SQL
SET search_path = pg_catalog, pg_temp;
SELECT format('SELECT %L, ''sqlx'', version, description, success, encode(checksum, ''hex'') FROM %I._sqlx_migrations ORDER BY version',
              '$db', n.nspname)
  FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE c.relname = '_sqlx_migrations' \gexec
SQL
  done
}

# Peta sistem -> database:pemilik:admin OpenBao:search_path role dinamis (§3.2, §3.4).
TARGET="app:ruang_sehat:rs_migration:openbao_rs_admin:app
audit:ruang_sehat_audit:rs_audit_migration:openbao_rs_audit_admin:audit
audit:rme_audit:rme_audit_migration:openbao_rme_audit_admin:audit
rme:rme:rme_migration:openbao_rme_admin:app"

# buat_dinamis <admin> <db> <nama> <pemilik> <search_path> <pw> <dengan_set_role:ya|tidak>
# Meniru creation_statements OpenBao (§3.4) dijalankan role admin OpenBao sungguhan. Nilai dikirim
# lewat stdin (bukan argumen proses yang terlihat di daftar proses); nama dan password dibatasi ke
# karakter aman lalu tetap dikutip format(%I/%L).
buat_dinamis() {
  local admin="$1" db="$2" nama="$3" pemilik="$4" sp="$5" pw="$6" setrole="$7"
  if ! printf '%s' "$nama$pemilik$sp" | grep -Eq '^[a-z0-9_-]+$' || ! printf '%s' "$pw" | grep -Eq '^[0-9a-f]+$'; then
    echo "buat_dinamis: nama/password di luar karakter aman" >&2
    return 1
  fi
  {
    echo "SET client_min_messages = warning;"
    echo "SELECT format('CREATE ROLE %I WITH LOGIN PASSWORD %L VALID UNTIL %L NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS IN ROLE %I', '$nama', '$pw', (now() + interval '30 minutes')::text, '$pemilik') \\gexec"
    if [ "$setrole" = "ya" ]; then
      echo "SELECT format('ALTER ROLE %I SET role = %L', '$nama', '$pemilik') \\gexec"
    fi
    echo "SELECT format('ALTER ROLE %I SET search_path = %I', '$nama', '$sp') \\gexec"
  } | psql_as "$admin" "$db"
}
hapus_dinamis() { # hapus_dinamis <admin> <db> <nama>: revocation_statements OpenBao (§3.4)
  psql_as "$1" "$2" -v nama="$3" <<'SQL'
SELECT format('DROP ROLE IF EXISTS %I', :'nama') \gexec
SQL
}

# jalankan_test <database> <berkas test> [opsi psql...]: lib.sql + berkas; setiap "LULUS" dihitung.
jalankan_test() {
  local db="$1" file="$2" out n baris
  shift 2
  if out="$(cat db/test/lib.sql "$file" | psql_su "$db" "$@" 2>&1)"; then
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
    [ -z "$RINCI" ] && echo "  $file ($db): $n pemeriksaan lulus"
    if [ "$n" -eq 0 ]; then
      tandai gagal "$file ($db): tidak ada pemeriksaan yang berjalan"
    fi
  else
    tandai gagal "$file ($db): $(printf '%s\n' "$out" | grep -E 'ERROR|GAGAL|FATAL' || printf '%s' "$out")"
  fi
}

# tolak_berkas <label> <pola galat> <role> <database> <berkas> [opsi psql tambahan sebelum -f...]
# Berkas migrasi dijalankan psql dalam satu transaksi sebagai role; WAJIB gagal dengan pola.
tolak_berkas() {
  local label="$1" pola="$2" role="$3" db="$4" file="$5" out
  shift 5
  if out="$(psql_as "$role" "$db" -1 "$@" -f - <"$file" 2>&1)"; then
    tandai gagal "NEG penjaga $label: berkas $(basename "$file") BERHASIL dijalankan, seharusnya ditolak"
  elif printf '%s\n' "$out" | grep -q -- "$pola"; then
    tandai lulus "NEG penjaga $label: $(basename "$file") ditolak ($pola)"
  else
    tandai gagal "NEG penjaga $label: $(basename "$file") ditolak dengan galat tak terduga: $out"
  fi
}

echo "db-migrations-check: sqlx-cli $VERSI_AKTIF ($SQLX)"
statis
statis_negatif

IMAGE="$(sed -n 's/^[[:space:]]*image:[[:space:]]*\(postgres:[^[:space:]]*@sha256:[0-9a-f]*\)[[:space:]]*$/\1/p' infra/local/compose.yaml | sed -n '1p')"
if [ -z "$IMAGE" ]; then
  echo "image postgres (tag@digest) tidak terbaca dari infra/local/compose.yaml" >&2
  exit 1
fi
CONTAINER="rs-db-migrations-check-$$-$RANDOM"
# Port hanya di loopback; password superuser acak sekali pakai (tidak dipakai: soket trust).
# Password lewat --env-file (mode 600 di direktori kerja), tidak terlihat di daftar proses.
( umask 077 && printf 'POSTGRES_PASSWORD=sekali-pakai-%s\n' "$(acak)" >"$KERJA/pg.env" )
docker run -d --rm --name "$CONTAINER" -p 127.0.0.1::5432 --env-file "$KERJA/pg.env" "$IMAGE" >/dev/null
rm -f "$KERJA/pg.env"
tunggu_siap
PORT="$(docker port "$CONTAINER" 5432/tcp | sed -n 's/^127\.0\.0\.1:\([0-9][0-9]*\)$/\1/p' | sed -n '1p')"
if [ -z "$PORT" ]; then
  echo "port PostgreSQL di 127.0.0.1 tidak terbaca dari docker port" >&2
  exit 1
fi
echo "  kontainer $CONTAINER ($IMAGE), 127.0.0.1:$PORT"

for f in db/roles/app.sql db/roles/audit.sql db/roles/rme.sql; do
  if out="$(psql_su postgres <"$f" 2>&1)"; then
    tandai lulus "bootstrap $f"
  else
    tandai gagal "bootstrap $f: $out"
  fi
done
if [ "$GAGAL" -ne 0 ]; then
  echo "db-migrations-check: $LULUS lulus, $GAGAL gagal (berhenti sebelum migrasi)"
  exit 1
fi

# ---------------------------------------------------------------------------------------------
# 3. Test negatif penjaga sebelum migrasi (role salah, urutan/target salah)
# ---------------------------------------------------------------------------------------------
potret >"$KERJA/potret-awal"
ROLE_TANPA_SET=""
negatif_penjaga() {
  local baris sistem db pemilik admin sp nama f app_role
  while IFS=: read -r sistem db pemilik admin sp; do
    nama="v-uji-tanpa-set-$db"
    buat_dinamis "$admin" "$db" "$nama" "$pemilik" "$sp" "$(acak)" tidak
    ROLE_TANPA_SET="$ROLE_TANPA_SET $admin:$db:$nama"
    case "$db" in
      ruang_sehat) app_role=rs_api ;;
      ruang_sehat_audit) app_role=rs_audit_writer_login ;;
      rme_audit) app_role=rme_audit_writer_login ;;
      rme) app_role=rme_api ;;
    esac
    for f in db/migrations/"$sistem"/*.sql; do
      [ -e "$f" ] || continue
      tolak_berkas "$db superuser" "migration must run as" postgres "$db" "$f"
      tolak_berkas "$db role aplikasi $app_role" "migration must run as" "$app_role" "$db" "$f"
      tolak_berkas "$db role dinamis tanpa SET role (mewarisi hak pemilik)" "migration must run as" "$nama" "$db" "$f"
    done
  done <<EOF
$TARGET
EOF
  # Urutan/target salah pada berkas pertama tiap direktori (bootstrap F1-T13 belum lengkap,
  # database salah, pemilik sistem lain). Dijalankan dalam satu transaksi yang dibatalkan galat.
  psql_su postgres -c 'CREATE DATABASE uji_urutan' >/dev/null
  f="$(find db/migrations/app -maxdepth 1 -name '*.sql' | sort | sed -n '1p')"
  if [ -n "$f" ]; then
    tolak_berkas "urutan: app sebelum bootstrap (database tanpa skema util)" "migration order: bootstrap db/roles/app.sql" \
      postgres uji_urutan "$f" -c 'SET LOCAL ROLE rs_migration'
    tolak_berkas "urutan: app saat skema public masih ada" "migration order: bootstrap db/roles/app.sql" \
      postgres ruang_sehat "$f" -c 'CREATE SCHEMA public' -c 'SET LOCAL ROLE rs_migration'
    tolak_berkas "target: app ke database audit" "migration must run as rs_migration" \
      postgres ruang_sehat_audit "$f" -c 'SET LOCAL ROLE rs_audit_migration'
  fi
  f="$(find db/migrations/audit -maxdepth 1 -name '*.sql' | sort | sed -n '1p')"
  if [ -n "$f" ]; then
    tolak_berkas "target: audit ke database bukan audit" "migration order: db/migrations/audit must run on" \
      postgres uji_urutan "$f" -c 'SET LOCAL ROLE rs_audit_migration'
    tolak_berkas "urutan: audit saat skema util belum milik pemilik" "migration order: bootstrap db/roles/audit.sql" \
      postgres ruang_sehat_audit "$f" -c 'ALTER SCHEMA util OWNER TO postgres' -c 'SET LOCAL ROLE rs_audit_migration'
    tolak_berkas "target: pemilik audit RME ke ruang_sehat_audit" "migration must run as rs_audit_migration" \
      postgres ruang_sehat_audit "$f" -c 'SET LOCAL ROLE rme_audit_migration'
    tolak_berkas "target: pemilik audit RS ke rme_audit" "migration must run as rme_audit_migration" \
      postgres rme_audit "$f" -c 'SET LOCAL ROLE rs_audit_migration'
  fi
  psql_su postgres -c 'DROP DATABASE uji_urutan' >/dev/null
  for baris in $ROLE_TANPA_SET; do
    IFS=: read -r admin db nama <<R
$baris
R
    hapus_dinamis "$admin" "$db" "$nama"
  done
}
negatif_penjaga
potret >"$KERJA/potret-sesudah-negatif"
if diff -u "$KERJA/potret-awal" "$KERJA/potret-sesudah-negatif" >/dev/null; then
  tandai lulus "NEG penjaga: katalog (role, keanggotaan, ACL, objek) tidak berubah oleh semua percobaan yang ditolak"
else
  tandai gagal "NEG penjaga: katalog berubah: $(diff -u "$KERJA/potret-awal" "$KERJA/potret-sesudah-negatif" || true)"
fi

# ---------------------------------------------------------------------------------------------
# 4. Migrasi dengan role dinamis tiruan, dua kali
# ---------------------------------------------------------------------------------------------
DINAMIS=""
migrasi() { # migrasi <putaran> <direktori sumber pengganti opsional>
  local putaran="$1" baris sistem db pemilik admin sp nama pw out sumber
  while IFS=: read -r sistem db pemilik admin sp; do
    nama="v-mig-$db-$(acak | cut -c1-8)"
    pw="$(acak)"
    if ! out="$(buat_dinamis "$admin" "$db" "$nama" "$pemilik" "$sp" "$pw" ya 2>&1)"; then
      tandai gagal "role dinamis $nama oleh $admin: $out"
      continue
    fi
    DINAMIS="$DINAMIS $admin:$db:$nama"
    sumber="$ROOT/db/migrations/$sistem"
    # Dijalankan dari direktori kerja kosong (tanpa .env / sqlx.toml repo), URL lewat lingkungan.
    if out="$(cd "$KERJA" && DATABASE_URL="postgres://$nama:$pw@127.0.0.1:$PORT/$db?sslmode=disable" \
      "$SQLX" migrate run --no-dotenv --source "$sumber" 2>&1)"; then
      if [ "$putaran" = 2 ] && printf '%s\n' "$out" | grep -q 'Applied'; then
        tandai gagal "idempoten: sqlx migrate run putaran 2 ($db) masih menerapkan migrasi: $out"
      else
        tandai lulus "sqlx migrate run putaran $putaran: db/migrations/$sistem -> $db sebagai $nama (current_user $pemilik) ($(printf '%s\n' "$out" | grep -c 'Applied' || true) diterapkan)"
      fi
    else
      tandai gagal "sqlx migrate run putaran $putaran: db/migrations/$sistem -> $db: $out"
    fi
  done <<EOF
$TARGET
EOF
}
migrasi 1
potret >"$KERJA/potret-migrasi-1"
migrasi 2
potret >"$KERJA/potret-migrasi-2"
if diff -u "$KERJA/potret-migrasi-1" "$KERJA/potret-migrasi-2" >/dev/null; then
  tandai lulus "idempoten: katalog dan _sqlx_migrations identik setelah sqlx migrate run kedua ($(wc -l <"$KERJA/potret-migrasi-1" | tr -d ' ') baris potret)"
else
  tandai gagal "idempoten: katalog berubah pada putaran 2: $(diff -u "$KERJA/potret-migrasi-1" "$KERJA/potret-migrasi-2" || true)"
fi
if grep -q "|sqlx|" "$KERJA/potret-migrasi-1"; then
  tandai lulus "_sqlx_migrations tercatat di skema bisnis tiap database ($(grep -c '|sqlx|' "$KERJA/potret-migrasi-1" | tr -d ' ') baris)"
else
  tandai gagal "_sqlx_migrations tidak terbaca di potret"
fi

# Berkas yang sudah diterapkan lalu diubah wajib ditolak checksum sqlx (§11.1).
checksum_negatif() {
  local f n sumber="$KERJA/ubah-app" nama pw out
  f="$(find db/migrations/app -maxdepth 1 -name '*.sql' | sort | sed -n '1p')"
  [ -n "$f" ] || return 0
  mkdir -p "$sumber"
  cp db/migrations/app/*.sql "$sumber/"
  printf '%s\n' "-- perubahan setelah diterapkan" >>"$sumber/$(basename "$f")"
  nama="v-mig-ubah-$(acak | cut -c1-8)"
  pw="$(acak)"
  buat_dinamis openbao_rs_admin ruang_sehat "$nama" rs_migration app "$pw" ya >/dev/null 2>&1
  DINAMIS="$DINAMIS openbao_rs_admin:ruang_sehat:$nama"
  n="${f##*/}"
  n="${n%%_*}"
  if out="$(cd "$KERJA" && DATABASE_URL="postgres://$nama:$pw@127.0.0.1:$PORT/ruang_sehat?sslmode=disable" \
    "$SQLX" migrate run --no-dotenv --source "$sumber" 2>&1)"; then
    tandai gagal "NEG checksum: berkas migrasi yang diubah setelah diterapkan diterima sqlx"
  elif printf '%s\n' "$out" | grep -q "migration $n was previously applied but has been modified"; then
    tandai lulus "NEG checksum: berkas $n yang diubah setelah diterapkan ditolak sqlx (previously applied but has been modified)"
  else
    tandai gagal "NEG checksum: galat tak terduga: $out"
  fi
}
checksum_negatif

# ---------------------------------------------------------------------------------------------
# 5. Test katalog per database
# ---------------------------------------------------------------------------------------------
if [ "$GAGAL" -eq 0 ]; then
  jalankan_test ruang_sehat db/test/katalog.sql -v skema_harap='{app,util,ext}' -v skema_proyek='{app,util}' -v skema_mig=app -v prefiks=rs_
  jalankan_test ruang_sehat db/test/util-app.sql
  jalankan_test ruang_sehat db/test/kelas-app.sql
  for p in rs_:ruang_sehat_audit rme_:rme_audit; do
    jalankan_test "${p#*:}" db/test/katalog.sql -v skema_harap='{audit,util,audit_detached}' \
      -v skema_proyek='{audit,util,audit_detached}' -v skema_mig=audit -v prefiks="${p%%:*}"
    jalankan_test "${p#*:}" db/test/util-audit.sql -v prefiks="${p%%:*}"
    jalankan_test "${p#*:}" db/test/kelas-audit.sql -v prefiks="${p%%:*}"
  done
  jalankan_test rme db/test/katalog.sql -v skema_harap='{app,util,ext}' -v skema_proyek='{app,util}' -v skema_mig=app -v prefiks=rme_
fi

# revocation_statements: DROP ROLE oleh admin OpenBao berhasil hanya bila role dinamis tidak
# memiliki objek apa pun (T-04, §3.4 review N02).
for baris in $DINAMIS; do
  IFS=: read -r admin db nama <<R
$baris
R
  if out="$(hapus_dinamis "$admin" "$db" "$nama" 2>&1)"; then
    tandai lulus "T-04 revocation_statements: DROP ROLE $nama oleh $admin berhasil (tidak memiliki objek)"
  else
    tandai gagal "T-04 revocation_statements: DROP ROLE $nama gagal: $out"
  fi
done
potret >"$KERJA/potret-akhir"
if diff <(grep -E '^(role|member|database)\|' "$KERJA/potret-awal") <(grep -E '^(role|member|database)\|' "$KERJA/potret-akhir") >/dev/null; then
  tandai lulus "T-05 role, atribut, keanggotaan, dan ACL database tidak diubah migrasi; role dinamis tidak tersisa"
else
  tandai gagal "T-05 role/keanggotaan/ACL database berubah oleh migrasi: $(diff <(grep -E '^(role|member|database)\|' "$KERJA/potret-awal") <(grep -E '^(role|member|database)\|' "$KERJA/potret-akhir") || true)"
fi
if psql_su postgres -At -c "SELECT count(*) FROM pg_roles WHERE rolname ~ '^v-'" | grep -qx 0; then
  tandai lulus "pembersihan: tidak ada role dinamis tersisa"
else
  tandai gagal "pembersihan: role dinamis tersisa"
fi

echo "db-migrations-check: $LULUS lulus, $GAGAL gagal"
if [ "$GAGAL" -ne 0 ] || [ "$LULUS" -eq 0 ]; then
  exit 1
fi
