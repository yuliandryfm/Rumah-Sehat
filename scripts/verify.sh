#!/usr/bin/env bash
# scripts/verify.sh: pintu verifikasi tunggal Ruang Sehat, definisi resmi "hijau".
# Dipakai git hook (pre-commit, pre-merge-commit), Stop hook Claude, dan CI.
# Rancangan: docs/05-stack.md §6 (KEPUTUSAN-82); pola: runbook Tahap 8.
#
# Aturan skrip ini:
# - Langkah sebuah komponen berjalan bila komponennya ada di repo, dan begitu ada wajib lulus.
# - Langkah yang belum diterapkan dicetak "BELUM DITERAPKAN (task)", tidak dihitung lulus,
#   dan jumlahnya disebut di ringkasan. Bila artefak komponennya sudah ada padahal langkahnya
#   belum diterapkan, langkah itu GAGAL (mencegah hijau palsu).
# - Tidak ada argumen, flag, atau variabel untuk melewati langkah.
# - Hijau berarti: nol langkah gagal dan minimal satu langkah lulus.
# - Setiap perubahan skrip ini meminta persetujuan user.
# - Portabel Linux dan macOS (bash 3.2): tanpa array asosiatif, tanpa `head`.
# - Pemindai rahasia dan dependensi (gitleaks, osv-scanner, cargo-deny, cargo-audit) wajib ada di PATH
#   dengan versi persis di bawah (docs/05-stack.md §1.6, §1.1); bila tidak ada atau versinya lain, langkahnya
#   GAGAL, tidak dilewati. Basis data advisory diambil dari jaringan setiap kali (tanpa mode offline).
# - Perilaku skrip ini diuji oleh scripts/test/test-verify.sh (fixture merah harus MERAH), dijalankan CI.

set -euo pipefail

if [ "$#" -ne 0 ]; then
  echo "verify.sh: tidak menerima argumen (diberikan: $*)." >&2
  exit 2
fi

# Antrean verifikasi (KEPUTUSAN-300): paling banyak 2 verifikasi penuh berjalan bersamaan per pengguna
# di satu mesin. Banyak worktree agent yang memverifikasi serentak membuat mesin 4 inti mencapai load
# 20-40 sehingga test berbasis waktu gagal palsu. Ini hanya MENUNDA giliran, tidak melewati atau
# melonggarkan langkah apa pun. Kunci dilepas otomatis saat proses selesai. Tanpa flock (macOS), tidak antre.
if command -v flock >/dev/null 2>&1; then
  ANTREAN_DIR="/tmp/ruang-sehat-verify-$(id -u)"
  mkdir -p "$ANTREAN_DIR"
  exec 8>"$ANTREAN_DIR/slot1"
  if ! flock -n 8; then
    exec 8>"$ANTREAN_DIR/slot2"
    if ! flock -n 8; then
      echo "verify.sh: menunggu giliran (paling banyak 2 verifikasi bersamaan)..." >&2
      flock 8
    fi
  fi
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Versi alat pemindai (docs/05-stack.md §1.6: SRC-151, SRC-152; §1.1: SRC-183, SRC-184).
# Harus sama dengan scripts/ci/pasang-alat.sh supaya CI memakai alat yang sama dengan lokal.
VERSI_GITLEAKS="8.30.1"
VERSI_OSV_SCANNER="2.7.1"
VERSI_CARGO_DENY="0.21.0"
VERSI_CARGO_AUDIT="0.23.0"

LULUS=0
GAGAL=0
BELUM=0
RINGKASAN=""

catat() { # catat <status> <keterangan>
  RINGKASAN="${RINGKASAN}$(printf '  %-17s %s' "$1" "$2")"$'\n'
}

jalankan() { # jalankan <nama> <perintah...>
  local nama="$1"
  shift
  echo ""
  echo "==> [$nama] $*"
  if "$@"; then
    LULUS=$((LULUS + 1))
    catat "LULUS" "$nama"
  else
    GAGAL=$((GAGAL + 1))
    catat "GAGAL" "$nama"
    echo "==> [$nama] GAGAL" >&2
  fi
}

gagal() { # gagal <nama> <alasan>
  echo ""
  echo "==> [$1] GAGAL: $2" >&2
  GAGAL=$((GAGAL + 1))
  catat "GAGAL" "$1: $2"
}

ada_pola() { # ada_pola <pola-glob...>: benar bila minimal satu berkas cocok
  local pola f
  for pola in "$@"; do
    for f in $pola; do
      [ -e "$f" ] && return 0
    done
  done
  return 1
}

belum() { # belum <nama> <task> [pola-glob artefak pemicu...]
  local nama="$1" task="$2"
  shift 2
  if [ "$#" -gt 0 ] && ada_pola "$@"; then
    gagal "$nama" "artefak komponen sudah ada ($*) tetapi langkah belum diterapkan ($task); perbarui scripts/verify.sh (perlu persetujuan user)"
    return 0
  fi
  echo ""
  echo "==> [$nama] BELUM DITERAPKAN ($task)"
  BELUM=$((BELUM + 1))
  catat "BELUM DITERAPKAN" "$nama ($task)"
}

periksa_toolchain() {
  local diminta aktif
  if [ ! -f rust-toolchain.toml ]; then
    echo "rust-toolchain.toml tidak ada" >&2
    return 1
  fi
  diminta="$(sed -n 's/^channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' rust-toolchain.toml | sed -n '1p')"
  aktif="$(rustc --version | sed -n '1p' | cut -d' ' -f2)"
  if [ -z "$diminta" ]; then
    echo "channel tidak terbaca dari rust-toolchain.toml" >&2
    return 1
  fi
  if [ "$aktif" != "$diminta" ]; then
    echo "rustc aktif $aktif, rust-toolchain.toml meminta $diminta" >&2
    return 1
  fi
  echo "rustc $aktif sesuai rust-toolchain.toml"
}

openapi_sinkron() { # F1-T11: spesifikasi di-commit harus identik dengan hasil generate (ADR-022 butir 3)
  local tmp f n status=0
  tmp="$(mktemp -d)" || return 1
  if ! tanpa_env_git cargo run --quiet --locked -p rs-backend --bin gen-openapi -- "$tmp"; then
    rm -rf "$tmp"
    return 1
  fi
  n=0
  for f in "$tmp"/*; do
    [ -e "$f" ] || continue
    n=$((n + 1))
    if ! cmp -s "$f" "openapi/$(basename "$f")"; then
      echo "openapi/$(basename "$f") berbeda dari hasil generate (atau belum ada); perbarui: cargo run -p rs-backend --bin gen-openapi -- openapi" >&2
      status=1
    fi
  done
  if [ "$n" -eq 0 ]; then
    echo "pembangkit tidak menghasilkan berkas" >&2
    status=1
  fi
  for f in openapi/*.json openapi/*.yaml openapi/*.yml; do
    [ -e "$f" ] || continue
    if [ ! -e "$tmp/$(basename "$f")" ]; then
      echo "$f tidak punya pembangkit (hapus berkas atau tambahkan pembangkitnya)" >&2
      status=1
    fi
  done
  rm -rf "$tmp"
  [ "$status" -eq 0 ] && echo "openapi/ identik dengan hasil generate ($n berkas)"
  return "$status"
}

# Schemathesis 4.33.4 (MIT) lewat image resmi Docker, dipin dengan digest (docs/sources.md SRC-382, SRC-383).
# Perkakas Python tidak dipasang di mesin; Docker wajib ada di mesin developer dan runner CI.
SCHEMATHESIS_IMAGE="schemathesis/schemathesis:4.33.4@sha256:708d934c8208a8f785bdf1b1610476e691f1efda0b7a3cb9ce7536d0f3280eec"
ST_PID=""
ST_TMP=""
ST_BAO=""
ST_BAO_ADDR=""

# F1-T61 (KEPUTUSAN-293): proses api memuat kunci blind index NIK dari OpenBao KV saat start (gagal cepat,
# pola email-relay) dan membungkus DEK lewat Transit. Backend efemeral Schemathesis karena itu memakai
# OpenBao dev sekali pakai (image yang sama dengan compose lokal, hanya 127.0.0.1, port acak): token root,
# kunci blind index, dan kunci Transit acak per proses, tidak pernah dicetak dan tidak lewat argumen
# perintah. Kontainer selalu dihapus openbao_api_bersih (dipanggil schemathesis_bersih, trap).
openbao_api_bersih() {
  if [ -n "$ST_BAO" ]; then
    docker rm -f "$ST_BAO" >/dev/null 2>&1 || true
    ST_BAO=""
  fi
  ST_BAO_ADDR=""
}

openbao_api_mulai() { # mengisi ST_BAO, ST_BAO_ADDR, dan ST_BAO_TOKEN (diekspor untuk proses api)
  local img port i siap=""
  img="$(sed -n 's|^[[:space:]]*image:[[:space:]]*\(openbao/openbao:[^[:space:]]*@sha256:[0-9a-f]*\)[[:space:]]*$|\1|p' \
    "$ROOT/infra/local/compose.yaml" | sed -n '1p')"
  if [ -z "$img" ]; then
    echo "image openbao (tag@digest) tidak terbaca dari infra/local/compose.yaml" >&2
    return 1
  fi
  ST_BAO_TOKEN="$(od -An -N24 -tx1 /dev/urandom | tr -d ' \n')"
  export ST_BAO_TOKEN
  ST_BAO="rs-st-openbao-$$-$RANDOM"
  if ! BAO_DEV_ROOT_TOKEN_ID="$ST_BAO_TOKEN" docker run -d --name "$ST_BAO" --cap-add IPC_LOCK \
    -e BAO_DEV_ROOT_TOKEN_ID -e BAO_DEV_LISTEN_ADDRESS=0.0.0.0:8200 -p 127.0.0.1::8200 \
    "$img" server -dev >/dev/null; then
    echo "OpenBao dev efemeral gagal dijalankan" >&2
    return 1
  fi
  i=0
  while [ "$i" -lt 60 ]; do
    if docker exec -e BAO_ADDR=http://127.0.0.1:8200 "$ST_BAO" bao status >/dev/null 2>&1; then
      siap=1
      break
    fi
    sleep 1
    i=$((i + 1))
  done
  # Kunci blind index versi 1 (32 byte acak, base64; bentuk KV crate crypto) dan kunci Transit rs-dek-kek
  # (aes256-gcm96, skema fisik §7.3), semuanya sintetis sekali pakai.
  if [ -z "$siap" ] || ! BAO_TOKEN="$ST_BAO_TOKEN" ST_BIDX_KEY="$(head -c 32 /dev/urandom | base64 | tr -d '\n')" \
    docker exec -e BAO_ADDR=http://127.0.0.1:8200 -e BAO_TOKEN -e ST_BIDX_KEY "$ST_BAO" sh -c \
      'bao secrets enable transit && bao write -f transit/keys/rs-dek-kek type=aes256-gcm96 && bao kv put secret/ruang-sehat/blind-index/nik 1="$ST_BIDX_KEY"' \
    >/dev/null; then
    echo "OpenBao dev efemeral tidak siap atau kunci sintetis tidak tertulis" >&2
    docker logs --tail 20 "$ST_BAO" >&2 || true
    return 1
  fi
  port="$(docker port "$ST_BAO" 8200/tcp | sed -n 's/^127\.0\.0\.1:\([0-9][0-9]*\)$/\1/p' | sed -n '1p')"
  if [ -z "$port" ]; then
    echo "port OpenBao dev efemeral tidak terbaca" >&2
    return 1
  fi
  ST_BAO_ADDR="http://127.0.0.1:$port"
}

schemathesis_bersih() { # dipasang sebagai trap: backend efemeral selalu dimatikan apa pun hasilnya
  if [ -n "$ST_PID" ]; then
    kill "$ST_PID" 2>/dev/null || true
    wait "$ST_PID" 2>/dev/null || true
    ST_PID=""
  fi
  openbao_api_bersih
  if [ -n "$ST_TMP" ]; then
    rm -rf "$ST_TMP"
    ST_TMP=""
  fi
}

schemathesis_run() { # contract test: backend efemeral di 127.0.0.1 (port acak) diuji terhadap tiap spesifikasi
  local bin bind i f n=0 status=0
  if ! command -v docker >/dev/null 2>&1; then
    echo "docker tidak ditemukan; Schemathesis dijalankan lewat image Docker" >&2
    return 1
  fi
  if ! docker image inspect "$SCHEMATHESIS_IMAGE" >/dev/null 2>&1; then
    if ! docker pull "$SCHEMATHESIS_IMAGE" >/dev/null; then
      echo "image $SCHEMATHESIS_IMAGE tidak tersedia" >&2
      return 1
    fi
  fi
  tanpa_env_git cargo build --quiet --locked -p rs-backend --bin rs-backend || return 1
  bin="${CARGO_TARGET_DIR:-target}/debug/rs-backend"
  if [ ! -x "$bin" ]; then
    echo "biner $bin tidak ditemukan" >&2
    return 1
  fi
  ST_TMP="$(mktemp -d)" || return 1
  # INT/TERM: bersihkan lalu keluar (130/143), jangan lanjut ke langkah berikutnya (review F1-T11).
  trap schemathesis_bersih EXIT
  trap 'schemathesis_bersih; exit 130' INT
  trap 'schemathesis_bersih; exit 143' TERM
  # Sejak F1-T92 proses api memakai database aplikasi: PostgreSQL sekali pakai dari scripts/db-ephemeral.sh
  # (login rs_api, hak runtime sungguhan, skema migrasi terbaru). Validasi token realm admin menunjuk JWKS di
  # alamat loopback yang tidak melayani, sehingga tidak ada token sah dan endpoint terautentikasi selalu
  # dijawab 401; object storage lokal dikonfigurasi tetapi tidak dijangkau. Data sintetis, tanpa data pribadi
  # (ADR-019); rahasia object storage acak sekali pakai. Sejak F1-T94 anggaran batas laju endpoint publik
  # dinaikkan untuk backend efemeral ini saja, supaya ribuan kasus Schemathesis menjangkau handler publik
  # (respons 429 tetap terdokumentasi dan diuji test integrasi); pemeriksaan Schemathesis tidak berubah.
  if [ -z "${RS_TEST_DB_APP_URL:-}" ]; then
    echo "RS_TEST_DB_APP_URL tidak ada: schemathesis_run wajib dijalankan lewat scripts/db-ephemeral.sh" >&2
    schemathesis_bersih
    trap - EXIT INT TERM
    return 1
  fi
  if ! openbao_api_mulai; then
    schemathesis_bersih
    trap - EXIT INT TERM
    return 1
  fi
  # API_VALKEY_URL menunjuk loopback yang tidak melayani: resolver akun memakai cadangan PostgreSQL (RT-18).
  # Realm pasien (F1-T61) sama seperti realm admin: JWKS di loopback yang tidak melayani, jadi endpoint
  # /patient/v1 selalu dijawab 401 di sini; skenario bertoken diuji test integrasi. Realm korporat (F1-T100)
  # sama: endpoint /v1/corporate selalu 401 di sini; isolasi tenant diuji tests/corporate_isolation.rs.
  API_DATABASE_URL="$RS_TEST_DB_APP_URL" \
    API_VALKEY_URL="redis://127.0.0.1:9" API_VALKEY_OP_TIMEOUT_MS=50 \
    AUTH_ADMIN_ISSUER="https://auth.ruang-sehat.invalid/realms/admin" \
    AUTH_ADMIN_JWKS_URL="http://127.0.0.1:9/realms/admin/protocol/openid-connect/certs" \
    AUTH_ADMIN_AUDIENCE="rs-backoffice-api" \
    AUTH_PATIENT_ISSUER="https://auth.ruang-sehat.invalid/realms/pasien" \
    AUTH_PATIENT_JWKS_URL="http://127.0.0.1:9/realms/pasien/protocol/openid-connect/certs" \
    AUTH_PATIENT_AUDIENCE="rs-patient-api" \
    AUTH_CORPORATE_ISSUER="https://auth.ruang-sehat.invalid/realms/korporat" \
    AUTH_CORPORATE_JWKS_URL="http://127.0.0.1:9/realms/korporat/protocol/openid-connect/certs" \
    AUTH_CORPORATE_AUDIENCE="rs-corporate-api" \
    OPENBAO_ADDR="$ST_BAO_ADDR" OPENBAO_TOKEN="$ST_BAO_TOKEN" \
    NIK_BLIND_INDEX_KV_PATH=ruang-sehat/blind-index/nik \
    API_DEK_CACHE_TTL_SECONDS=300 API_DEK_CACHE_CAPACITY=64 \
    DEPLOY_ENV=local OBJECT_STORAGE_ENDPOINT="http://127.0.0.1:9" OBJECT_STORAGE_REGION=us-east-1 \
    OBJECT_STORAGE_ADDRESSING=path OBJECT_STORAGE_ACCESS_KEY_ID=schemathesis-sintetis \
    OBJECT_STORAGE_SECRET_ACCESS_KEY="st-$(od -An -N24 -tx1 /dev/urandom | tr -d ' \n')" \
    OBJECT_STORAGE_BUCKET_PREFIX=rs-st- \
    PUBLIC_RATE_LIMIT_PER_SOURCE_PER_MINUTE=1000000 PUBLIC_RATE_LIMIT_GLOBAL_PER_MINUTE=1000000 \
    HTTP_BIND=127.0.0.1:0 LOG_LEVEL=info "$bin" api >"$ST_TMP/backend.log" 2>&1 &
  ST_PID=$!
  bind=""
  i=0
  while [ "$i" -lt 200 ]; do
    bind="$(sed -n 's/.*"message":"api_listening".*"bind":"\([^"]*\)".*/\1/p' "$ST_TMP/backend.log" | sed -n '1p')"
    [ -n "$bind" ] && break
    if ! kill -0 "$ST_PID" 2>/dev/null; then
      break
    fi
    sleep 0.1
    i=$((i + 1))
  done
  if [ -z "$bind" ]; then
    echo "backend efemeral tidak siap; log:" >&2
    cat "$ST_TMP/backend.log" >&2
    schemathesis_bersih
    trap - EXIT INT TERM
    return 1
  fi
  echo "backend efemeral di http://$bind"
  for f in openapi/*.json openapi/*.yaml openapi/*.yml; do
    [ -e "$f" ] || continue
    n=$((n + 1))
    echo "-- schemathesis: $f"
    # Pemeriksaan: tanpa unsupported_method dan allow_header_conformance karena registry
    # docs/05-response-codes.md tidak memuat 405 (metode tak didukung dijawab 404, KU-03).
    if ! docker run --rm --network host --workdir /tmp -v "$ROOT/openapi:/openapi:ro" "$SCHEMATHESIS_IMAGE" \
      run "/openapi/$(basename "$f")" --url "http://$bind" --no-color --seed 1 --max-examples 50 \
      --checks not_a_server_error,status_code_conformance,content_type_conformance,response_headers_conformance,response_schema_conformance,positive_data_acceptance,negative_data_rejection; then
      status=1
    fi
  done
  if [ "$n" -eq 0 ]; then
    echo "tidak ada spesifikasi di openapi/ untuk diuji" >&2
    status=1
  fi
  schemathesis_bersih
  trap - EXIT INT TERM
  return "$status"
}

schemathesis_dengan_db() { # F1-T92: schemathesis_run di dalam PostgreSQL sekali pakai (scripts/db-ephemeral.sh)
  # Fungsi dan variabel yang dipakai schemathesis_run diekspor ke shell anak yang dijalankan db-ephemeral;
  # kontainer database selalu dihapus db-ephemeral (trap), backend efemeral oleh schemathesis_bersih.
  export -f schemathesis_run schemathesis_bersih tanpa_env_git openbao_api_mulai openbao_api_bersih
  export ROOT SCHEMATHESIS_IMAGE ST_PID ST_TMP ST_BAO ST_BAO_ADDR
  tanpa_env_git bash scripts/db-ephemeral.sh bash -c schemathesis_run
}

pakai_sqlx() { # benar bila workspace sudah memakai sqlx
  [ -d .sqlx ] && return 0
  [ -f Cargo.lock ] && grep -q '^name = "sqlx"$' Cargo.lock
}

sqlx_prepare_check() { # F1-T23: .sqlx identik dengan metadata hasil generate terhadap skema migrasi terbaru
  # PostgreSQL sekali pakai + migrasi db/migrations/app dari scripts/db-ephemeral.sh; DATABASE_URL = role
  # pemilik skema tiruan (hanya untuk describe query). --all-targets: query di test ikut diperiksa.
  # sqlx-cli 0.9.1 hanya memperingatkan berkas .sqlx yang tidak dipakai lagi; di sini peringatan itu GAGAL
  # supaya metadata usang tidak tertinggal (SRC-490).
  local log status=0
  log="$(mktemp)" || return 1
  tanpa_env_git bash scripts/db-ephemeral.sh sh -c \
    'DATABASE_URL="$RS_TEST_DB_OWNER_URL" exec cargo sqlx prepare --check --workspace -- --all-targets --locked' \
    2>&1 | tee "$log" || status=1
  if [ "$status" -eq 0 ] && grep -q 'potentially unused queries found in .sqlx' "$log"; then
    echo "berkas .sqlx usang (query tidak dipakai lagi); perbarui: bash scripts/db-ephemeral.sh sh -c 'DATABASE_URL=\"\$RS_TEST_DB_OWNER_URL\" cargo sqlx prepare --workspace -- --all-targets --locked'" >&2
    status=1
  fi
  rm -f "$log"
  return "$status"
}

tanpa_env_git() { # tanpa_env_git <perintah...>: jalankan perintah tanpa variabel GIT_* warisan git hook
  # Hook pre-commit mengeset GIT_INDEX_FILE ke index repo ini. Alat yang menjalankan git di repo lain
  # ikut memakainya: cargo-audit 0.23.0 yang mengambil advisory-db menimpa index repo ini dengan index
  # advisory-db (direproduksi saat F1-T06). Hanya git dan gitleaks yang memakai GIT_* warisan hook.
  local v hapus
  hapus=()
  for v in $(env | sed -n 's/^\(GIT_[A-Za-z0-9_]*\)=.*/\1/p'); do
    hapus+=(-u "$v")
  done
  env ${hapus[@]+"${hapus[@]}"} "$@"
}

versi_alat() { # versi_alat <biner>: cetak nomor versi alat pemindai
  case "$1" in
    gitleaks) gitleaks version ;;
    osv-scanner) osv-scanner --version | sed -n 's/^osv-scanner version: //p' ;;
    cargo-deny) cargo-deny --version | sed -n 's/^cargo-deny //p' ;;
    cargo-audit) cargo-audit --version | sed -n 's/^cargo-audit //p' ;;
    *) return 1 ;;
  esac
}

alat_siap() { # alat_siap <biner> <versi>: alat ada di PATH dan versinya persis sama
  local aktif
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 tidak ditemukan di PATH (dibutuhkan versi $2, docs/05-stack.md)" >&2
    return 1
  fi
  aktif="$(versi_alat "$1" 2>/dev/null | sed -n '1p' || true)"
  if [ "$aktif" != "$2" ]; then
    echo "$1 aktif '${aktif}', dibutuhkan $2 (docs/05-stack.md)" >&2
    return 1
  fi
  echo "$1 $aktif ($(command -v "$1"))"
}

periksa_cargo_deny() { # lisensi, advisory, larangan, dan sumber crate menurut deny.toml
  alat_siap cargo-deny "$VERSI_CARGO_DENY" || return 1
  if [ ! -f deny.toml ]; then
    echo "deny.toml tidak ada" >&2
    return 1
  fi
  tanpa_env_git cargo-deny --locked --config deny.toml check
}

periksa_cargo_audit() { # advisory RustSec atas Cargo.lock; peringatan (unmaintained, unsound, yanked) = gagal
  alat_siap cargo-audit "$VERSI_CARGO_AUDIT" || return 1
  if [ ! -f Cargo.lock ]; then
    echo "Cargo.lock tidak ada" >&2
    return 1
  fi
  tanpa_env_git cargo-audit audit --file Cargo.lock --deny warnings
}

konfigurasi_liar() { # konfigurasi_liar <nama-berkas> <path-resmi>: cetak berkas bernama sama di luar path resmi
  # Semua berkas di pohon kerja: dilacak, belum dilacak, dan yang diabaikan (repo bersarang tidak ditelusuri).
  git ls-files --cached --others | grep -E "(^|/)$(printf '%s' "$1" | sed 's/[.]/[.]/g')\$" | grep -vxF "$2" || true
}

periksa_gitleaks() { # rahasia di riwayat HEAD, perubahan ter-stage, dan perubahan belum di-stage
  local liar
  alat_siap gitleaks "$VERSI_GITLEAKS" || return 1
  # Konfigurasi resmi diberikan eksplisit (--config/--gitleaks-ignore-path) supaya .gitleaks.toml lain tidak
  # dimuat otomatis; berkas .gitleaks.toml selain yang resmi ditolak (review keamanan F1-T06, B-1).
  if [ ! -f .gitleaks.toml ] || [ ! -f .gitleaksignore ]; then
    echo ".gitleaks.toml atau .gitleaksignore resmi di akar repo tidak ada" >&2
    return 1
  fi
  liar="$(konfigurasi_liar .gitleaks.toml .gitleaks.toml)"
  if [ -n "$liar" ]; then
    echo "konfigurasi gitleaks di luar berkas resmi ditolak: $(printf '%s' "$liar" | tr '\n' ' ')" >&2
    return 1
  fi
  # --ignore-gitleaks-allow: komentar "gitleaks:allow" tidak berlaku; pengecualian hanya lewat .gitleaksignore.
  echo "-- riwayat commit yang dapat dicapai dari HEAD"
  gitleaks git --no-banner --redact --ignore-gitleaks-allow --config .gitleaks.toml --gitleaks-ignore-path .gitleaksignore --log-opts="--full-history HEAD" . || return 1
  echo "-- perubahan ter-stage"
  gitleaks git --no-banner --redact --ignore-gitleaks-allow --config .gitleaks.toml --gitleaks-ignore-path .gitleaksignore --pre-commit --staged . || return 1
  echo "-- perubahan belum di-stage"
  gitleaks git --no-banner --redact --ignore-gitleaks-allow --config .gitleaks.toml --gitleaks-ignore-path .gitleaksignore --pre-commit . || return 1
}

periksa_osv_scanner() { # kerentanan pada setiap lockfile yang dilacak git (atau belum dilacak, tidak diabaikan)
  local berkas opsi liar asing
  alat_siap osv-scanner "$VERSI_OSV_SCANNER" || return 1
  # Konfigurasi resmi diberikan eksplisit (--config) supaya osv-scanner.toml di direktori lockfile tidak dimuat
  # otomatis; osv-scanner.toml selain yang resmi ditolak (review keamanan F1-T06, B-1).
  if [ ! -f osv-scanner.toml ]; then
    echo "osv-scanner.toml resmi di akar repo tidak ada" >&2
    return 1
  fi
  liar="$(konfigurasi_liar osv-scanner.toml osv-scanner.toml)"
  if [ -n "$liar" ]; then
    echo "konfigurasi osv-scanner di luar berkas resmi ditolak: $(printf '%s' "$liar" | tr '\n' ' ')" >&2
    return 1
  fi
  # Berkas berpola lockfile yang tidak dikenali tidak boleh lolos tanpa dipindai (review F1-T06, non-blokir 3).
  asing="$(git ls-files --cached --others --exclude-standard \
    | grep -E '(^|/)([^/]*\.lockb?|[^/]*lock\.json|[^/]*lock\.ya?ml|[^/]*\.lockfile|go\.sum|npm-shrinkwrap\.json|Package\.resolved)$' \
    | grep -vE '(^|/)(Cargo\.lock|pnpm-lock\.yaml|package-lock\.json|yarn\.lock|pubspec\.lock)$' || true)"
  if [ -n "$asing" ]; then
    echo "lockfile yang tidak dikenali verify.sh: $(printf '%s' "$asing" | tr '\n' ' '); perbarui scripts/verify.sh (perlu persetujuan user)" >&2
    return 1
  fi
  # Daftar eksplisit dari git, bukan pemindaian rekursif: pemindai rekursif ikut menghormati .gitignore
  # di direktori induk repo (misalnya checkout utama yang mengabaikan .claude/worktrees/).
  opsi=()
  while IFS= read -r berkas; do
    if [ -f "$berkas" ]; then
      opsi+=(--lockfile "$berkas")
    fi
  done < <(git ls-files --cached --others --exclude-standard | grep -E '(^|/)(Cargo\.lock|pnpm-lock\.yaml|package-lock\.json|yarn\.lock|pubspec\.lock)$' || true)
  if [ "${#opsi[@]}" -eq 0 ]; then
    echo "tidak ada lockfile yang dapat dipindai" >&2
    return 1
  fi
  tanpa_env_git osv-scanner scan source --all-vulns --config osv-scanner.toml "${opsi[@]}"
}

echo "verify.sh: $ROOT"

# --- Langkah 0: lingkungan tidak boleh mengubah hasil (review keamanan F1-T05, B-1) ---
# Variabel berikut dapat mematikan lint atau mengganti runner test sehingga hasil hijau palsu.
# Ditolak (GAGAL), tidak di-unset diam-diam. GITLEAKS_CONFIG(_TOML) dapat mengganti aturan gitleaks.
# Konfigurasi di luar repo (~/.cargo/config.toml, ~/.cargo/audit.toml, .cargo/config.toml di direktori
# induk repo) ditutup oleh CI: runner bersih + scripts/ci/periksa-lingkungan-bersih.sh (F1-T06).
VAR_TERLARANG="$(env | grep -E '^(RUSTFLAGS|CARGO_ENCODED_RUSTFLAGS|RUSTDOCFLAGS|RUSTC|RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER|CARGO_TARGET_[A-Za-z0-9_]*_RUNNER|CARGO_ALIAS_[A-Za-z0-9_]*|CARGO_BUILD_[A-Za-z0-9_]*|CARGO_PROFILE_[A-Za-z0-9_]*|GITLEAKS_CONFIG|GITLEAKS_CONFIG_TOML)=' | cut -d= -f1 | tr '\n' ' ' || true)"
if [ -n "$VAR_TERLARANG" ]; then
  gagal "lingkungan" "variabel yang dapat mengubah hasil verifikasi diset: ${VAR_TERLARANG}; hapus dari lingkungan lalu jalankan ulang"
fi
if [ -e .cargo/config ] || [ -e .cargo/config.toml ]; then
  gagal "lingkungan" ".cargo/config(.toml) ada di repo tetapi belum disetujui sebagai bagian verifikasi; perbarui scripts/verify.sh (perlu persetujuan user)"
fi
# .cargo/audit.toml dimuat otomatis oleh cargo-audit dan dapat mengabaikan advisory (SRC-361; review F1-T06, B-1).
if [ -e .cargo/audit.toml ]; then
  gagal "lingkungan" ".cargo/audit.toml ada di repo tetapi belum disetujui sebagai bagian verifikasi; perbarui scripts/verify.sh (perlu persetujuan user)"
fi

# Makro query sqlx selalu memakai metadata offline .sqlx yang di-commit (coding-standards §5.2), bukan
# database dari DATABASE_URL atau .env developer: nilai lingkungan/berkas .env diabaikan dengan menimpa
# kedua variabel ini (variabel proses didahulukan atas .env, SRC-490). Langkah sqlx-prepare menjalankan
# `cargo check` dengan SQLX_OFFLINE=false dan direktori sementara yang diset sqlx-cli sendiri.
export SQLX_OFFLINE=true
export SQLX_OFFLINE_DIR="$ROOT/.sqlx"

# --- Langkah 1: Rust (docs/05-stack.md §6 butir 1) ---
if [ ! -f Cargo.toml ]; then
  gagal "rust" "Cargo.toml (workspace, KEPUTUSAN-89) tidak ada"
elif ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
  gagal "rust" "cargo/rustc tidak ditemukan di PATH"
else
  jalankan "rust-toolchain" periksa_toolchain
  jalankan "cargo-fmt" cargo fmt --all --check
  # clippy dan test menjalankan build script dan test, yang dapat memanggil git: tanpa GIT_* warisan hook.
  jalankan "cargo-clippy" tanpa_env_git cargo clippy --workspace --all-targets --locked -- -D warnings
  # Sejak F1-T23 test integrasi memakai PostgreSQL 18 sekali pakai (scripts/db-ephemeral.sh, butuh docker);
  # test yang butuh database GAGAL bila variabel RS_TEST_DB_* tidak ada, tidak dilewati.
  jalankan "cargo-test" tanpa_env_git bash scripts/db-ephemeral.sh cargo test --workspace --locked
fi

if pakai_sqlx; then
  jalankan "sqlx-prepare" sqlx_prepare_check
else
  belum "sqlx-prepare" "task pertama yang memakai sqlx; belum ditetapkan di phase plan §7.3"
fi
jalankan "cargo-deny" periksa_cargo_deny
jalankan "cargo-audit" periksa_cargo_audit

# --- Langkah 2: Migrasi dan hak role (§6 butir 2; KEPUTUSAN-140, KEPUTUSAN-158) ---
# 2a. Bootstrap klaster db/roles/ (F1-T13): dijalankan dua kali (idempoten) ke PostgreSQL 18 sekali
#     pakai (butuh docker), lalu test katalog dan test negatif hak role (skema fisik §3.5, §11.1, §12).
#     Sejak F1-T13 berkas bootstrap wajib ada; hilangnya berkas itu GAGAL.
if ada_pola "db/roles/*.sql"; then
  jalankan "bootstrap-role-db" tanpa_env_git bash scripts/db-roles-check.sh ephemeral
else
  gagal "bootstrap-role-db" "db/roles/*.sql (bootstrap klaster, F1-T13) tidak ada"
fi
# 2b. Migrasi + test katalog (F1-T15): pemeriksaan teks berkas migrasi (penjaga, maju saja, T-23),
#     PostgreSQL 18 sekali pakai (image sama dengan 2a), bootstrap, test negatif penjaga, sqlx-cli
#     0.9.1 `migrate run` per database sebagai role migrasi dinamis tiruan (dua kali, idempoten),
#     lalu test katalog db/test/*.sql (skema fisik §11, §12). Sejak F1-T15 langkah ini wajib.
if [ -f scripts/db-migrations-check.sh ]; then
  # tanpa_env_git: runner memakai `git merge-base`/`git show`; GIT_INDEX_FILE dkk. warisan hook
  # pre-merge-commit membuat fixture statis salah dibaca (merge F1-T06, 2026-11-19).
  jalankan "migrasi-db" tanpa_env_git bash scripts/db-migrations-check.sh
else
  gagal "migrasi-db" "scripts/db-migrations-check.sh (F1-T15) tidak ada"
fi

# --- Langkah 3: Kontrak API (§6 butir 3) ---
if [ -f apps/rs-backend/Cargo.toml ] && command -v cargo >/dev/null 2>&1; then
  jalankan "openapi-sinkron" openapi_sinkron
  # Setiap response_code di spesifikasi terdaftar di docs/05-response-codes.md (envelope::registry::parse),
  # deskripsinya sama, dan tiga digit pertamanya sama dengan HTTP respons. --test gagal bila targetnya hilang.
  jalankan "registry-response-code" tanpa_env_git cargo test --locked -p rs-backend --test openapi_response_codes
else
  gagal "openapi-sinkron" "apps/rs-backend tidak ada atau cargo tidak ditemukan"
  gagal "registry-response-code" "apps/rs-backend tidak ada atau cargo tidak ditemukan"
fi
belum "asyncapi-sinkron" "task AsyncAPI belum ditetapkan" "openapi/asyncapi/*.yaml" "openapi/asyncapi/*.yml" "openapi/asyncapi/*.json"
if [ -f apps/rs-backend/Cargo.toml ] && command -v cargo >/dev/null 2>&1; then
  jalankan "schemathesis" schemathesis_dengan_db
else
  gagal "schemathesis" "apps/rs-backend tidak ada atau cargo tidak ditemukan"
fi

# --- Langkah 4: Test arsitektur (§6 butir 4) ---
# F1-T12: crate arch-check memeriksa kepemilikan tabel modul (apps/*/module-tables.toml), konfigurasi
# lint tidak dilemahkan (workspace.lints, clippy.toml root), jam sistem/env/aws-lc-rs langsung, pola log
# terlarang, dan pengecualian sempit (architecture-exceptions.toml), lengkap dengan uji negatifnya.
# Sejak F1-T12 berkas module-tables.toml wajib ada; hilangnya GAGAL.
if ada_pola "apps/*/module-tables.toml"; then
  jalankan "test-arsitektur" tanpa_env_git cargo test --locked -p arch-check
else
  gagal "test-arsitektur" "apps/*/module-tables.toml (F1-T12) tidak ada"
fi

# --- Langkah 5: Web (§6 butir 5) ---
# F1-T39: workspace pnpm di web/ (Node 26, pnpm 12.21.0, TypeScript ketat, ESLint, Vitest). Versi dipin di
# web/package.json dan docs/sources.md (SRC-500 s.d. SRC-509). Begitu web/ ada, semua langkah wajib lulus.
web_alat_siap() { # node mayor 26 dan pnpm persis sama dengan packageManager di web/package.json
  local pm_diminta pm_aktif node_aktif
  if ! command -v node >/dev/null 2>&1 || ! command -v pnpm >/dev/null 2>&1; then
    echo "node dan pnpm harus ada di PATH (docs/05-stack.md §1.2)" >&2
    return 1
  fi
  node_aktif="$(node --version | sed -n '1p')"
  case "$node_aktif" in
    v26.*) ;;
    *)
      echo "node aktif $node_aktif, dibutuhkan 26.x (docs/05-stack.md §1.2)" >&2
      return 1
      ;;
  esac
  pm_diminta="$(sed -n 's/^[[:space:]]*"packageManager"[[:space:]]*:[[:space:]]*"pnpm@\([^"]*\)".*/\1/p' web/package.json | sed -n '1p')"
  pm_aktif="$(cd web && pnpm --version | sed -n '1p')"
  if [ -z "$pm_diminta" ] || [ "$pm_aktif" != "$pm_diminta" ]; then
    echo "pnpm aktif '${pm_aktif}', web/package.json meminta '${pm_diminta}'" >&2
    return 1
  fi
  echo "node $node_aktif, pnpm $pm_aktif"
}

web_dependensi_segar() { # hijau palsu karena node_modules basi (pola npm ci di tahap-08-development.md)
  # pnpm menulis salinan lockfile ke node_modules/.pnpm/lock.yaml dalam format berbeda dari pnpm-lock.yaml
  # (tanpa dokumen lingkungan di awal), jadi bukan perbandingan byte. Penjaganya: `pnpm install
  # --frozen-lockfile` dijalankan setiap kali (langkah web-install; gagal bila package.json dan lockfile
  # tidak sinkron dan memasang ulang node_modules yang tidak sesuai lockfile), ditambah pemeriksaan di bawah
  # bahwa keadaan pemasangan ada dan setiap workspace yang berdependensi tercatat di salinan lockfile terpasang.
  if [ ! -f web/pnpm-lock.yaml ]; then
    echo "web/pnpm-lock.yaml tidak ada (lockfile wajib di-commit)" >&2
    return 1
  fi
  if ! git ls-files --error-unmatch web/pnpm-lock.yaml >/dev/null 2>&1; then
    # Sebelum commit pertama lockfile boleh baru di-stage atau belum dilacak; hook commit memverifikasi index.
    if [ -z "$(git ls-files --others --exclude-standard web/pnpm-lock.yaml)" ]; then
      echo "web/pnpm-lock.yaml tidak dilacak git" >&2
      return 1
    fi
  fi
  if [ ! -f web/node_modules/.pnpm/lock.yaml ] || [ ! -f web/node_modules/.modules.yaml ]; then
    echo "web/node_modules belum terpasang dari lockfile (keadaan pemasangan pnpm tidak ada)" >&2
    return 1
  fi
  # Setiap workspace yang dideklarasikan harus muncul sebagai importer di salinan lockfile terpasang.
  local dir
  for dir in . backoffice portal rme situs korporat packages/*; do
    [ -f "web/$dir/package.json" ] || continue
    # pnpm hanya mencatat importer yang punya dependensi.
    grep -qE '"(dependencies|devDependencies)"[[:space:]]*:' "web/$dir/package.json" || continue
    if ! grep -qE "^  $(printf '%s' "$dir" | sed 's/[.\/]/\\&/g'):\$" web/node_modules/.pnpm/lock.yaml; then
      echo "web/$dir tidak terdaftar sebagai importer di web/node_modules/.pnpm/lock.yaml (node_modules basi)" >&2
      return 1
    fi
  done
  echo "web/node_modules terpasang dari lockfile; semua workspace terdaftar"
}

di_web() { # di_web <perintah...>: jalankan di web/ tanpa variabel GIT_* warisan hook
  (cd web && tanpa_env_git "$@")
}

web_build() { # build setiap paket/aplikasi web yang punya skrip build
  di_web pnpm run build
}

if [ -f web/package.json ] && [ -f web/pnpm-workspace.yaml ]; then
  jalankan "web-alat" web_alat_siap
  jalankan "web-install" di_web pnpm install --frozen-lockfile
  jalankan "web-dependensi-segar" web_dependensi_segar
  jalankan "web-typecheck" di_web pnpm run typecheck
  jalankan "web-lint" di_web pnpm run lint
  jalankan "web-test" di_web pnpm run test
  jalankan "web-api-client-sinkron" di_web pnpm run generate:check
  # TD-32 (KEPUTUSAN-136/297/298): lisensi dependensi npm runtime dan dev sesuai daftar izin; Unknown = gagal.
  if [ -f web/scripts/periksa-lisensi.mjs ]; then
    jalankan "web-lisensi" di_web pnpm run lisensi
  else
    gagal "web-lisensi" "web/scripts/periksa-lisensi.mjs (TD-32) tidak ada"
  fi
  # F1-T40: skrip kontras token tanpa dependensi (K1–K13, N1–N4; design guideline §7.4), lalu build
  # setiap paket/aplikasi yang punya skrip build (saat ini galeri state web/packages/ui; aplikasi Next.js
  # menambah `next build` pada task fiturnya). Hilangnya semua skrip build GAGAL, bukan dilewati.
  if [ -f web/packages/ui/scripts/kontras.mjs ]; then
    jalankan "web-kontras" di_web pnpm run kontras
  else
    gagal "web-kontras" "web/packages/ui/scripts/kontras.mjs (F1-T40) tidak ada"
  fi
  if grep -l '"build"[[:space:]]*:' web/*/package.json web/packages/*/package.json >/dev/null 2>&1; then
    jalankan "web-build" web_build
  else
    gagal "web-build" "tidak ada paket web dengan skrip build (web/packages/ui, F1-T40)"
  fi
elif [ -e web/package.json ] || [ -e web/pnpm-workspace.yaml ]; then
  gagal "web" "web/package.json dan web/pnpm-workspace.yaml harus ada bersamaan"
else
  gagal "web" "workspace web/ (F1-T39) tidak ada"
fi

# --- Langkah 6: Mobile (§6 butir 6) ---
belum "mobile" "F1-T44" "mobile/pubspec.yaml" "mobile/apps/*/pubspec.yaml" "mobile/packages/*/pubspec.yaml"

# --- Langkah 7: Rahasia dan dependensi (§6 butir 7, §3.2) ---
jalankan "gitleaks" periksa_gitleaks
jalankan "osv-scanner" periksa_osv_scanner

# --- Ringkasan ---
echo ""
echo "== Ringkasan verify.sh =="
printf '%s' "$RINGKASAN"
echo "Lulus: $LULUS, gagal: $GAGAL, belum diterapkan: $BELUM (tidak dihitung lulus)"
if [ "$GAGAL" -ne 0 ]; then
  echo "HASIL: MERAH ($GAGAL langkah gagal)"
  exit 1
fi
if [ "$LULUS" -eq 0 ]; then
  echo "HASIL: MERAH (tidak ada langkah yang berjalan)"
  exit 1
fi
echo "HASIL: HIJAU ($LULUS langkah lulus, $BELUM langkah belum diterapkan)"
