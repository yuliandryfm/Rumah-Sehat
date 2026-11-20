#!/usr/bin/env bash
# scripts/db-ephemeral.sh: PostgreSQL 18 sekali pakai berisi skema database aplikasi (ruang_sehat) untuk
# test integrasi Rust dan `cargo sqlx prepare --check` (F1-T23). Dipakai scripts/verify.sh langkah
# cargo-test dan sqlx-prepare; dapat juga dijalankan manual.
#
# Pola sama dengan scripts/db-migrations-check.sh (F1-T15): image postgres dipin tag@digest dari
# infra/local/compose.yaml, port hanya di 127.0.0.1 (acak), bootstrap db/roles/app.sql, migrasi
# db/migrations/app dengan sqlx-cli 0.9.1 sebagai role migrasi dinamis tiruan OpenBao (bukan superuser).
# Kontainer dan direktori kerja selalu dihapus (trap). Data sintetis saja (ADR-019). Portabel bash 3.2.
#
# Pakai: bash scripts/db-ephemeral.sh <perintah> [argumen...]
# Perintah dijalankan dengan variabel (password acak sekali pakai, hanya untuk kontainer ini):
#   RS_TEST_DB_APP_URL    login rs_api (anggota rs_app): hak runtime sungguhan
#   RS_TEST_DB_RELAY_URL  login rs_audit_relay_login (anggota rs_audit_relay)
#   RS_TEST_DB_OWNER_URL  role dinamis dengan SET role = rs_migration (pemilik skema): untuk metadata
#                         sqlx (DATABASE_URL `cargo sqlx prepare`), bukan untuk test hak akses
# Kode keluar = kode keluar perintah; galat penyiapan = 1.

set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "pakai: bash scripts/db-ephemeral.sh <perintah> [argumen...]" >&2
  exit 2
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SQLX_VERSI="0.9.1"
SQLX_ROOT="$HOME/.local/share/ruang-sehat/sqlx-cli-$SQLX_VERSI"

CONTAINER=""
KERJA="$(mktemp -d)"
bersihkan() {
  if [ -n "$CONTAINER" ]; then
    docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
  fi
  rm -rf "$KERJA"
}
trap bersihkan EXIT
trap 'bersihkan; exit 130' INT
trap 'bersihkan; exit 143' TERM

for alat in docker od; do
  if ! command -v "$alat" >/dev/null 2>&1; then
    echo "db-ephemeral: $alat tidak ditemukan di PATH" >&2
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
  echo "db-ephemeral: sqlx-cli $SQLX_VERSI tidak ditemukan (di $SQLX_ROOT/bin maupun PATH: ${VERSI_AKTIF:-tidak ada})" >&2
  echo "  pasang: cargo install sqlx-cli --version $SQLX_VERSI --locked --no-default-features --features postgres,rustls --root $SQLX_ROOT" >&2
  exit 1
fi
# cargo sqlx (subperintah cargo) dari instalasi yang sama didahulukan di PATH.
PATH="$(dirname "$SQLX"):$PATH"
export PATH

acak() { od -An -N16 -tx1 /dev/urandom | tr -d ' \n'; }

psql_su() { # psql_su <database> [opsi psql...] (SQL dari stdin), superuser lewat soket lokal
  local db="$1"
  shift
  docker exec -i -u postgres "$CONTAINER" psql -X -q -v ON_ERROR_STOP=1 -U postgres -d "$db" "$@"
}
psql_as() { # psql_as <role> <database> (SQL dari stdin): login sungguhan lewat soket lokal (trust)
  docker exec -i -u postgres "$CONTAINER" psql -X -q -v ON_ERROR_STOP=1 -U "$1" -d "$2"
}

IMAGE="$(sed -n 's/^[[:space:]]*image:[[:space:]]*\(postgres:[^[:space:]]*@sha256:[0-9a-f]*\)[[:space:]]*$/\1/p' infra/local/compose.yaml | sed -n '1p')"
if [ -z "$IMAGE" ]; then
  echo "db-ephemeral: image postgres (tag@digest) tidak terbaca dari infra/local/compose.yaml" >&2
  exit 1
fi
CONTAINER="rs-db-ephemeral-$$-$RANDOM"
# Password superuser lewat --env-file (mode 600), tidak terlihat di daftar proses; tidak dipakai (soket trust).
(umask 077 && printf 'POSTGRES_PASSWORD=sekali-pakai-%s\n' "$(acak)" >"$KERJA/pg.env")
docker run -d --rm --name "$CONTAINER" -p 127.0.0.1::5432 --env-file "$KERJA/pg.env" "$IMAGE" >/dev/null
rm -f "$KERJA/pg.env"
siap=""
for _ in $(seq 1 90); do
  if docker exec "$CONTAINER" pg_isready -q -h 127.0.0.1 -U postgres >/dev/null 2>&1; then
    siap=ya
    break
  fi
  sleep 1
done
if [ -z "$siap" ]; then
  echo "db-ephemeral: PostgreSQL di kontainer $CONTAINER tidak siap dalam 90 detik" >&2
  exit 1
fi
PORT="$(docker port "$CONTAINER" 5432/tcp | sed -n 's/^127\.0\.0\.1:\([0-9][0-9]*\)$/\1/p' | sed -n '1p')"
if [ -z "$PORT" ]; then
  echo "db-ephemeral: port PostgreSQL di 127.0.0.1 tidak terbaca dari docker port" >&2
  exit 1
fi

if ! out="$(psql_su postgres <db/roles/app.sql 2>&1)"; then
  echo "db-ephemeral: bootstrap db/roles/app.sql gagal: $out" >&2
  exit 1
fi

# Role migrasi dinamis tiruan (skema fisik §3.4), dibuat role admin OpenBao sungguhan; nilai lewat stdin.
OWNER="v-ephemeral-owner-$(acak | cut -c1-8)"
OWNER_PW="$(acak)"
if ! out="$(
  {
    echo "SET client_min_messages = warning;"
    echo "SELECT format('CREATE ROLE %I WITH LOGIN PASSWORD %L VALID UNTIL %L NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS IN ROLE %I', '$OWNER', '$OWNER_PW', (now() + interval '6 hours')::text, 'rs_migration') \\gexec"
    echo "SELECT format('ALTER ROLE %I SET role = %L', '$OWNER', 'rs_migration') \\gexec"
    echo "SELECT format('ALTER ROLE %I SET search_path = %I', '$OWNER', 'app') \\gexec"
  } | psql_as openbao_rs_admin ruang_sehat 2>&1
)"; then
  echo "db-ephemeral: pembuatan role migrasi dinamis gagal: $out" >&2
  exit 1
fi
OWNER_URL="postgres://$OWNER:$OWNER_PW@127.0.0.1:$PORT/ruang_sehat?sslmode=disable"
if ! out="$(cd "$KERJA" && DATABASE_URL="$OWNER_URL" "$SQLX" migrate run --no-dotenv --source "$ROOT/db/migrations/app" 2>&1)"; then
  echo "db-ephemeral: sqlx migrate run db/migrations/app gagal: $out" >&2
  exit 1
fi

# Password sekali pakai untuk role login runtime (bootstrap sengaja tanpa password, KEPUTUSAN-79).
APP_PW="$(acak)"
RELAY_PW="$(acak)"
# Nilai lewat stdin (bukan argumen proses yang terlihat di daftar proses); hanya karakter heksadesimal.
if ! out="$(
  {
    echo "SET client_min_messages = warning;"
    echo "SELECT format('ALTER ROLE rs_api PASSWORD %L', '$APP_PW') \\gexec"
    echo "SELECT format('ALTER ROLE rs_audit_relay_login PASSWORD %L', '$RELAY_PW') \\gexec"
  } | psql_su postgres 2>&1
)"; then
  echo "db-ephemeral: pengaturan password role login gagal: $out" >&2
  exit 1
fi

export RS_TEST_DB_APP_URL="postgres://rs_api:$APP_PW@127.0.0.1:$PORT/ruang_sehat?sslmode=disable"
export RS_TEST_DB_RELAY_URL="postgres://rs_audit_relay_login:$RELAY_PW@127.0.0.1:$PORT/ruang_sehat?sslmode=disable"
export RS_TEST_DB_OWNER_URL="$OWNER_URL"
echo "db-ephemeral: $IMAGE di 127.0.0.1:$PORT, migrasi db/migrations/app diterapkan" >&2

status=0
"$@" || status=$?
exit "$status"
