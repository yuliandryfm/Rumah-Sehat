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
#      negatif pemeriksa dengan berkas rusak buatan. Pemeriksaan teks dapat diakali SQL dinamis
#      yang lebih rumit; kontrol jangka panjang adalah event trigger (KEPUTUSAN-158) + T-08.
#   1b. Urutan terhadap basis git (merge-base dengan main/tahap-08-fase-1): berkas basis identik,
#      berkas baru berversi lebih besar; penjaga DO berkas lanjutan menolak bila _sqlx_migrations
#      sudah memuat versi lebih besar (templat db/test/templat-penjaga-lanjutan-*.sql).
#      Butir untuk task skrip deploy production (review DBA 3c): skrip deploy wajib menolak
#      menerapkan berkas yang versinya lebih kecil dari versi tertinggi di _sqlx_migrations target
#      dan berkas yang checksum-nya berbeda, sebelum memanggil sqlx.
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
# tanpa_komentar: buang komentar SQL -- (sampai akhir baris) dan /* */ (bersarang) dari stdin.
# Literal '...' (dengan '' sebagai kutip di dalamnya) dilewati utuh; isi dollar-quote diperlakukan
# sebagai kode (badan DO/fungsi), sehingga komentar di dalamnya ikut dibuang. Karena penyederhanaan
# ini, pola juga dicari di teks mentah (gabungan keduanya).
tanpa_komentar() {
  awk '{ s = s $0 "\n" }
    END {
      n = length(s); d = 0; q = 0; i = 1; out = ""
      while (i <= n) {
        ch = substr(s, i, 1); c = substr(s, i, 2)
        if (q) { out = out ch; i++; if (ch == "\047") q = 0; continue }
        if (d == 0 && ch == "\047") { q = 1; out = out ch; i++; continue }
        if (d == 0 && c == "--") { while (i <= n && substr(s, i, 1) != "\n") i++; out = out " "; continue }
        if (c == "/*") { d++; i += 2; continue }
        if (d > 0 && c == "*/") { d--; i += 2; if (d == 0) out = out " "; continue }
        if (d > 0) { i++; continue }
        out = out ch; i++
      }
      printf "%s", out
    }'
}

# temuan_statis <direktori> <app|audit|rme>: mencetak satu baris per temuan (kosong = bersih).
temuan_statis() {
  local dir="$1" sistem="$2" f nama penjaga datar bersih teks lit versi semua_versi="" pertama
  pertama="$(find "$dir" -maxdepth 1 -name '*.sql' -exec basename {} \; | sort | sed -n '1p')"
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
    # Penjaga urutan berkas lanjutan (review DBA F1-T14/T15 butir 3b): setiap berkas selain berkas
    # pertama direktori menolak dijalankan bila _sqlx_migrations sudah memuat versi lebih besar;
    # literal versi di penjaga wajib sama dengan versi nama berkas.
    if [ "$nama" != "$pertama" ]; then
      if ! printf '%s\n' "$penjaga" | grep -Eq "_sqlx_migrations WHERE version > $versi\)"; then
        echo "$nama: penjaga lanjutan tanpa pemeriksaan urutan '_sqlx_migrations WHERE version > $versi)' (literal = versi nama berkas)"
      fi
      if printf '%s\n' "$penjaga" | grep -Eo '_sqlx_migrations WHERE version > [0-9]+' | grep -Evq " $versi\$"; then
        echo "$nama: literal versi penjaga urutan berbeda dari versi nama berkas $versi"
      fi
    fi
    # T-23 (§11.4, KEPUTUSAN-158) dan pengerasan review keamanan F1-T14/T15: pola dicari pada teks
    # mentah DAN teks tanpa komentar (-- dan /* */ bersarang) dengan spasi dinormalkan, sehingga
    # komentar di antara kata kunci tidak meloloskan pola. Literal string (setelah penggabungan
    # 'a' || 'b', tanpa literal COMMENT ON ... IS) diperiksa untuk SQL dinamis. Ini pemeriksaan teks
    # yang dapat diakali dengan SQL dinamis yang lebih rumit (chr(), format('%s')); kontrol jangka
    # panjang adalah event trigger (KEPUTUSAN-158) ditambah test katalog T-08 setelah migrasi.
    datar="$(tr '\n\t\r' '   ' <"$f" | tr -s ' ')"
    bersih="$(tanpa_komentar <"$f" | tr '\n\t\r' '   ' | tr -s ' ')"
    for teks in "$datar" "$bersih"; do
      if printf '%s\n' "$teks" | grep -Eiq 'DISABLE TRIGGER'; then echo "$nama: memuat DISABLE TRIGGER (T-23, §11.4)"; fi
      if printf '%s\n' "$teks" | grep -Eiq 'ENABLE (REPLICA|ALWAYS) TRIGGER'; then echo "$nama: memuat ENABLE REPLICA/ALWAYS TRIGGER (trigger tidak terpicu pada sesi biasa, T-23)"; fi
      if printf '%s\n' "$teks" | grep -Eiq '(DROP|ALTER) TRIGGER (IF EXISTS )?"?tg_[a-z0-9_]*_(reject_[a-z0-9_]*|guard_delete|one_way|effective_lock|no_backdate|published_lock|single_source)\b'; then
        echo "$nama: memuat DROP/ALTER TRIGGER atas trigger penjaga (T-23, §11.4)"
      fi
      if printf '%s\n' "$teks" | grep -Eiq 'replication_role'; then echo "$nama: memuat session_replication_role (mematikan trigger, T-23)"; fi
      if printf '%s\n' "$teks" | grep -Eiq 'set_config'; then echo "$nama: memuat set_config (pengaturan sesi dinamis dilarang di migrasi)"; fi
      if printf '%s\n' "$teks" | grep -Eiq 'DROP FUNCTION (IF EXISTS )?"?util"?\.'; then echo "$nama: memuat DROP FUNCTION util.* (CASCADE melepas trigger penjaga)"; fi
      if printf '%s\n' "$teks" | grep -Eiq '(DROP|ALTER) SCHEMA'; then echo "$nama: memuat DROP/ALTER SCHEMA (skema dikelola bootstrap db/roles)"; fi
      if ! printf '%s\n' "$nama" | grep -Eq '^[0-9]{14}_util_[a-z0-9_]+\.sql$' &&
        printf '%s\n' "$teks" | grep -Eiq '(ALTER FUNCTION|CREATE OR REPLACE FUNCTION) "?util"?\.'; then
        echo "$nama: mengubah fungsi util di luar berkas *_util_*.sql (§11.4: penggantian fungsi penjaga hanya di migrasi util yang direview)"
      fi
    done
    while IFS= read -r lit; do
      [ -n "$lit" ] || continue
      if printf '%s\n' "$lit" | grep -Eiq 'ABLE TRIGGER|REPLICA TRIGGER|ALWAYS TRIGGER|(DROP|ALTER) TRIGGER|(DROP|ALTER) FUNCTION|(DROP|ALTER) SCHEMA|replication_role'; then
        echo "$nama: literal string SQL dinamis memuat perintah terlarang: $lit"
      elif printf '%s\n' "$lit" | grep -Eiq 'trigger|util\.' && ! printf '%s\n' "$lit" | grep -Eiq "^'[[:space:]]*CREATE (OR REPLACE )?TRIGGER "; then
        echo "$nama: literal string memuat TRIGGER/util. di luar teks CREATE TRIGGER: $lit"
      fi
    done <<LIT
$(printf '%s\n' "$bersih" | sed -E "s/COMMENT ON [^;']* IS '([^']|'')*'/COMMENT/g" |
  sed -E ":a
s/'(([^']|'')*)' *\|\| *'(([^']|'')*)'/'\1\3'/
ta" | grep -Eo "'([^']|'')*'" || true)
LIT
  done
}

statis() {
  local sistem t
  for sistem in app audit rme; do
    t="$(temuan_statis "db/migrations/$sistem" "$sistem" | sort -u)"
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
  kasus_lanjutan() { # kasus_lanjutan <label> <isi berkas kedua> <pola temuan>
    no=$((no + 1))
    d="$KERJA/statis-neg-$no"
    mkdir -p "$d"
    printf '%s\n' "$PENJAGA_APP" >"$d/20990101000000_util_awal.sql"
    printf '%s\n' "$2" >"$d/20990101000002_lanjutan.sql"
    t="$(temuan_statis "$d" app)"
    if printf '%s\n' "$t" | grep -q -- "$3"; then
      tandai lulus "NEG statis: $1"
    else
      tandai gagal "NEG statis: $1 tidak terdeteksi (temuan: ${t:-kosong})"
    fi
  }
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
  # Bentuk pengakalan T-23 dari review keamanan F1-T14/T15 (Medium).
  kasus "DISABLE/**/TRIGGER" app 20990101000000_k1.sql "$PENJAGA_APP
ALTER TABLE app.x DISABLE/**/TRIGGER ALL;" "DISABLE TRIGGER"
  kasus "DISABLE -- komentar baris TRIGGER" app 20990101000000_k2.sql "$PENJAGA_APP
ALTER TABLE app.x DISABLE -- c
TRIGGER ALL;" "DISABLE TRIGGER"
  kasus "EXECUTE 'DIS'||'ABLE TRIGGER ALL'" app 20990101000000_k3.sql "$PENJAGA_APP
DO \$x\$ BEGIN EXECUTE 'ALTER TABLE app.x DIS'||'ABLE TRIGGER ALL'; END \$x\$;" "literal string SQL dinamis"
  kasus "DROP TRIGGER/**/tg_x_reject_ud" app 20990101000000_k4.sql "$PENJAGA_APP
DROP TRIGGER/**/tg_x_reject_ud ON app.x;" "DROP/ALTER TRIGGER atas trigger penjaga"
  kasus "ENABLE REPLICA TRIGGER" app 20990101000000_k5.sql "$PENJAGA_APP
ALTER TABLE app.x ENABLE REPLICA TRIGGER tg_x_reject_ud;" "ENABLE REPLICA/ALWAYS TRIGGER"
  kasus "DROP FUNCTION util.fn_reject_modification() CASCADE" app 20990101000000_k6.sql "$PENJAGA_APP
DROP FUNCTION util.fn_reject_modification() CASCADE;" "DROP FUNCTION util"
  kasus "set_config('session_'||'replication_role', ...)" app 20990101000000_k7.sql "$PENJAGA_APP
SELECT set_config('session_'||'replication_role', 'replica', true);" "set_config"
  kasus "literal gabungan replication_role (tanpa set_config)" app 20990101000000_k8.sql "$PENJAGA_APP
DO \$x\$ BEGIN EXECUTE 'SET LOCAL session_'||'replication_role = replica'; END \$x\$;" "literal string SQL dinamis"
  kasus "CREATE OR REPLACE FUNCTION util di berkas bukan util" app 20990101000000_tabel_x.sql "$PENJAGA_APP
CREATE OR REPLACE FUNCTION util.fn_reject_modification() RETURNS trigger LANGUAGE plpgsql AS \$f\$ BEGIN RETURN NULL; END \$f\$;" "di luar berkas \*_util_\*.sql"
  kasus "ALTER FUNCTION util di berkas bukan util" app 20990101000000_tabel_y.sql "$PENJAGA_APP
ALTER FUNCTION util.fn_guard_log_delete() SECURITY DEFINER;" "di luar berkas \*_util_\*.sql"
  kasus "DROP SCHEMA util CASCADE" app 20990101000000_k9.sql "$PENJAGA_APP
DROP SCHEMA util CASCADE;" "DROP/ALTER SCHEMA"
  kasus "format() dengan literal DROP TRIGGER" app 20990101000000_k10.sql "$PENJAGA_APP
DO \$x\$ BEGIN EXECUTE format('DROP TRIGGER %I ON app.x', 'tg_x_' || 'reject_ud'); END \$x\$;" "literal string SQL dinamis"
  kasus "literal util. di luar CREATE TRIGGER" app 20990101000000_k11.sql "$PENJAGA_APP
DO \$x\$ BEGIN EXECUTE 'COMMENT ON FUNCTION ' || 'util.fn_x() IS NULL'; END \$x\$;" "TRIGGER/util. di luar teks CREATE TRIGGER"
  # Penjaga urutan berkas lanjutan (butir 3b): dua berkas, berkas kedua rusak.
  kasus_lanjutan "berkas lanjutan tanpa pemeriksaan urutan" "$PENJAGA_APP" "penjaga lanjutan tanpa pemeriksaan urutan"
  kasus_lanjutan "literal versi penjaga urutan berbeda dari nama berkas" \
    "$(sed 's/@VERSI@/20990101000001/g' db/test/templat-penjaga-lanjutan-app.sql)" "literal versi penjaga urutan berbeda"
  # Tanpa positif palsu: templat lanjutan resmi dan pola partisi §6.2 (format CREATE TRIGGER util.*).
  no=$((no + 1))
  d="$KERJA/statis-bersih-$no"
  mkdir -p "$d"
  printf '%s\n' "$PENJAGA_APP" >"$d/20990101000000_util_awal.sql"
  { sed 's/@VERSI@/20990101000002/g' db/test/templat-penjaga-lanjutan-app.sql
    cat <<'SQL'
-- Pola §6.2: trigger penolak per partisi lewat SQL dinamis (sah).
COMMENT ON TABLE app.x IS 'class:T module:uji -- Trigger penolak';
DO $$
BEGIN
  EXECUTE format('CREATE TRIGGER %I BEFORE UPDATE OR DELETE ON app.%I FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification()', 'tg_x_reject_ud', 'x');
END $$;
SQL
  } >"$d/20990101000002_tabel_x.sql"
  t="$(temuan_statis "$d" app)"
  if [ -z "$t" ]; then
    tandai lulus "statis tanpa positif palsu: templat penjaga lanjutan + format('CREATE TRIGGER ... util.fn_reject_modification()') + COMMENT berisi 'Trigger'"
  else
    tandai gagal "statis positif palsu: $t"
  fi
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
SELECT 'paramacl', parname, paracl FROM pg_parameter_acl ORDER BY parname;
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

# ---------------------------------------------------------------------------------------------
# 1b. Penjaga urutan terhadap basis git (review DBA F1-T14/T15 butir 3a): berkas migrasi yang
#     sudah ada di basis (git merge-base HEAD main, atau tahap-08-fase-1 bila basis main belum
#     memuat berkas migrasi) wajib identik byte demi byte di pohon kerja (tidak diubah, diganti
#     nama, atau dihapus), dan versi berkas baru wajib lebih besar dari versi tertinggi di basis.
# ---------------------------------------------------------------------------------------------
# temuan_basis <direktori basis> <direktori kini>: satu baris per temuan (kosong = bersih).
temuan_basis() {
  local basis="$1" kini="$2" f nama maks="" v
  for f in "$basis"/*.sql; do
    [ -e "$f" ] || continue
    nama="$(basename "$f")"
    v="${nama%%_*}"
    if [ -z "$maks" ] || [ "$v" \> "$maks" ]; then maks="$v"; fi
    if [ ! -e "$kini/$nama" ]; then
      echo "$nama: ada di basis tetapi hilang (dihapus/diganti nama; berkas yang sudah diterapkan tidak boleh berubah, §11.1)"
    elif ! cmp -s "$f" "$kini/$nama"; then
      echo "$nama: berbeda dari basis (berkas yang sudah diterapkan tidak boleh diubah; koreksi lewat berkas baru, §11.1)"
    fi
  done
  for f in "$kini"/*.sql; do
    [ -e "$f" ] || continue
    nama="$(basename "$f")"
    [ -e "$basis/$nama" ] && continue
    v="${nama%%_*}"
    if [ -n "$maks" ] && ! [ "$v" \> "$maks" ]; then
      echo "$nama: berkas baru dengan versi $v tidak lebih besar dari versi tertinggi di basis $maks (urutan maju saja)"
    fi
  done
}

basis_git() {
  local ref mb sistem p t n ada_ref=""
  if ! command -v git >/dev/null 2>&1 || ! git rev-parse --git-dir >/dev/null 2>&1; then
    tandai gagal "basis git: bukan repositori git atau git tidak ada (penjaga urutan butuh riwayat)"
    return 0
  fi
  BASIS=""
  BASIS_KET=""
  for ref in main tahap-08-fase-1; do
    git rev-parse -q --verify "$ref^{commit}" >/dev/null 2>&1 || continue
    mb="$(git merge-base HEAD "$ref" 2>/dev/null)" || continue
    ada_ref="$ref"
    if git ls-tree -r --name-only "$mb" -- db/migrations | grep -q '\.sql$'; then
      BASIS="$mb"
      BASIS_KET="merge-base HEAD $ref ($(printf '%s' "$mb" | cut -c1-10))"
      break
    fi
  done
  if [ -z "$ada_ref" ]; then
    tandai gagal "basis git: ref main maupun tahap-08-fase-1 tidak ditemukan (ambil riwayat lengkap, mis. git fetch origin main:main)"
    return 0
  fi
  for sistem in app audit rme; do
    mkdir -p "$KERJA/basis/$sistem"
    if [ -n "$BASIS" ]; then
      for p in $(git ls-tree -r --name-only "$BASIS" -- "db/migrations/$sistem" | grep '\.sql$' || true); do
        git show "$BASIS:$p" >"$KERJA/basis/$sistem/$(basename "$p")"
      done
    fi
    n="$(find "$KERJA/basis/$sistem" -name '*.sql' | wc -l | tr -d ' ')"
    t="$(temuan_basis "$KERJA/basis/$sistem" "db/migrations/$sistem")"
    if [ -n "$t" ]; then
      tandai gagal "urutan terhadap basis db/migrations/$sistem: $t"
    elif [ -n "$BASIS" ]; then
      tandai lulus "urutan terhadap basis db/migrations/$sistem: $n berkas basis identik, berkas baru berversi lebih besar ($BASIS_KET)"
    else
      tandai lulus "urutan terhadap basis db/migrations/$sistem: basis (merge-base dengan main/tahap-08-fase-1) belum memuat berkas migrasi; semua berkas baru"
    fi
  done
}

basis_negatif() {
  local no=0 b k t
  kasus_basis() { # kasus_basis <label> <pola> ; berkas disiapkan di $b dan $k oleh pemanggil
    t="$(temuan_basis "$b" "$k")"
    if [ -n "$2" ]; then
      if printf '%s\n' "$t" | grep -q -- "$2"; then tandai lulus "NEG basis: $1"; else tandai gagal "NEG basis: $1 tidak terdeteksi (temuan: ${t:-kosong})"; fi
    else
      if [ -z "$t" ]; then tandai lulus "basis: $1"; else tandai gagal "basis: $1 menghasilkan temuan: $t"; fi
    fi
  }
  siapkan() {
    no=$((no + 1))
    b="$KERJA/basis-neg-$no/basis"
    k="$KERJA/basis-neg-$no/kini"
    mkdir -p "$b" "$k"
    printf 'SELECT 1;\n' >"$b/20990101000000_awal.sql"
    printf 'SELECT 2;\n' >"$b/20990102000000_kedua.sql"
    cp "$b"/*.sql "$k/"
  }
  siapkan; printf 'SELECT 3;\n' >"$k/20990103000000_ketiga.sql"
  kasus_basis "berkas basis identik + berkas baru berversi lebih besar" ""
  siapkan; printf -- '-- ubah\n' >>"$k/20990102000000_kedua.sql"
  kasus_basis "berkas yang sudah ada di basis diubah" "berbeda dari basis"
  siapkan; mv "$k/20990101000000_awal.sql" "$k/20990101000000_awal_baru.sql"
  kasus_basis "berkas basis diganti nama" "hilang (dihapus/diganti nama"
  siapkan; rm "$k/20990102000000_kedua.sql"
  kasus_basis "berkas basis dihapus" "hilang (dihapus/diganti nama"
  siapkan; printf 'SELECT 4;\n' >"$k/20990101120000_sisipan.sql"
  kasus_basis "berkas baru berversi di bawah versi tertinggi basis (sisipan)" "tidak lebih besar"
  siapkan; printf 'SELECT 5;\n' >"$k/20990102000000_kembar.sql"
  kasus_basis "berkas baru berversi sama dengan versi tertinggi basis" "tidak lebih besar"
  no=$((no + 1))
  b="$KERJA/basis-neg-$no/basis"
  k="$KERJA/basis-neg-$no/kini"
  mkdir -p "$b" "$k"
  printf 'SELECT 1;\n' >"$k/20990101000000_awal.sql"
  kasus_basis "basis tanpa berkas migrasi: semua berkas baru diterima" ""
}

echo "db-migrations-check: sqlx-cli $VERSI_AKTIF ($SQLX)"
statis
statis_negatif
basis_git
basis_negatif

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

# Penjaga urutan di dalam berkas lanjutan (review DBA butir 3b), dengan templat resmi
# db/test/templat-penjaga-lanjutan-*.sql: berkas fixture bervesi tinggi diterapkan, lalu berkas
# fixture dengan versi lebih kecil (sqlx menerapkannya tanpa menolak, SRC-399) wajib ditolak
# penjaganya. Dijalankan setelah test katalog; fixture hanya berisi penjaga dan SELECT 1.
urutan_dinamis() {
  local sistem db pemilik admin sp tmpl sumber nama pw out
  for p in app:ruang_sehat:rs_migration:openbao_rs_admin:app audit:ruang_sehat_audit:rs_audit_migration:openbao_rs_audit_admin:audit; do
    IFS=: read -r sistem db pemilik admin sp <<R
$p
R
    tmpl="db/test/templat-penjaga-lanjutan-$sistem.sql"
    sumber="$KERJA/urut-$sistem"
    mkdir -p "$sumber"
    cp db/migrations/"$sistem"/*.sql "$sumber/"
    { sed 's/@VERSI@/29991231000000/g' "$tmpl"; echo "SELECT 1;"; } >"$sumber/29991231000000_uji_urutan_tinggi.sql"
    nama="v-mig-urut-$(acak | cut -c1-8)"
    pw="$(acak)"
    buat_dinamis "$admin" "$db" "$nama" "$pemilik" "$sp" "$pw" ya >/dev/null 2>&1
    DINAMIS="$DINAMIS $admin:$db:$nama"
    if out="$(cd "$KERJA" && DATABASE_URL="postgres://$nama:$pw@127.0.0.1:$PORT/$db?sslmode=disable" \
      "$SQLX" migrate run --no-dotenv --source "$sumber" 2>&1)"; then
      tandai lulus "penjaga urutan ($db): berkas lanjutan dari templat diterima pada urutan benar"
    else
      tandai gagal "penjaga urutan ($db): berkas lanjutan dari templat ditolak padahal urutan benar: $out"
      continue
    fi
    { sed 's/@VERSI@/29991230000000/g' "$tmpl"; echo "SELECT 1;"; } >"$sumber/29991230000000_uji_urutan_mundur.sql"
    if out="$(cd "$KERJA" && DATABASE_URL="postgres://$nama:$pw@127.0.0.1:$PORT/$db?sslmode=disable" \
      "$SQLX" migrate run --no-dotenv --source "$sumber" 2>&1)"; then
      tandai gagal "NEG penjaga urutan ($db): berkas berversi lebih kecil dari migrasi terapan DITERIMA"
    elif printf '%s\n' "$out" | grep -q "migration order: version 29991230000000 is older than an applied migration"; then
      tandai lulus "NEG penjaga urutan ($db): berkas 29991230000000 setelah 29991231000000 diterapkan ditolak penjaganya"
    else
      tandai gagal "NEG penjaga urutan ($db): galat tak terduga: $out"
    fi
  done
}
if [ "$GAGAL" -eq 0 ]; then
  urutan_dinamis
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
if diff <(grep -E '^(role|member|database|paramacl)\|' "$KERJA/potret-awal") <(grep -E '^(role|member|database|paramacl)\|' "$KERJA/potret-akhir") >/dev/null; then
  tandai lulus "T-05 role, atribut, keanggotaan, ACL database, dan ACL parameter (pg_parameter_acl) tidak diubah migrasi; role dinamis tidak tersisa"
else
  tandai gagal "T-05 role/keanggotaan/ACL database/parameter berubah oleh migrasi: $(diff <(grep -E '^(role|member|database|paramacl)\|' "$KERJA/potret-awal") <(grep -E '^(role|member|database|paramacl)\|' "$KERJA/potret-akhir") || true)"
fi
# pg_parameter_acl (review keamanan butir Low): tidak ada role proyek dengan hak SET/ALTER SYSTEM
# atas parameter, dan potret mendeteksi GRANT SET ON PARAMETER session_replication_role.
if psql_su postgres -At -c "SELECT count(*) FROM pg_parameter_acl p, aclexplode(p.paracl) a WHERE pg_get_userbyid(a.grantee) ~ '^(rs|rme|openbao)_'" | grep -qx 0; then
  tandai lulus "T-05 pg_parameter_acl: tidak ada role proyek dengan hak atas parameter server"
else
  tandai gagal "T-05 pg_parameter_acl: ada role proyek dengan hak atas parameter server"
fi
psql_su postgres -c 'GRANT SET ON PARAMETER session_replication_role TO rs_migration' >/dev/null
potret >"$KERJA/potret-param"
psql_su postgres -c 'REVOKE SET ON PARAMETER session_replication_role FROM rs_migration' >/dev/null
if grep -E '^paramacl\|session_replication_role\|' "$KERJA/potret-param" | grep -q 'rs_migration=s'; then
  tandai lulus "NEG T-05 potret mendeteksi GRANT SET ON PARAMETER session_replication_role TO rs_migration"
else
  tandai gagal "NEG T-05 GRANT SET ON PARAMETER tidak terlihat di potret: $(grep '^paramacl' "$KERJA/potret-param" || echo kosong)"
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
