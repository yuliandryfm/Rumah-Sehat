#!/usr/bin/env bash
# scripts/keycloak-check.sh: menerapkan realm Keycloak sebagai kode (infra/keycloak/realms/*.json) dengan
# keycloak-config-cli, lalu menjalankan test konfigurasi dan perilaku ke Keycloak sungguhan (F1-T31; klien
# layanan pengambil event berhak minimum: F1-T34).
#
# Modul kritis T1 (ADR-011, CLAUDE.md). Dasar: docs/05-stack.md §7, KEPUTUSAN-85, KEPUTUSAN-190,
# KEPUTUSAN-199, KEPUTUSAN-201, KEPUTUSAN-203, KEPUTUSAN-231, ADR-026.
#
# Pakai:
#   bash scripts/keycloak-check.sh compose [rinci]
#       Keycloak compose lokal yang sedang berjalan (infra/local/compose.yaml, layanan `keycloak`, sehat).
#       Konfigurasi diterapkan lewat layanan `keycloak-config` (profil `alat`) dengan nilai dari
#       infra/local/.env. GAGAL bila Keycloak tidak berjalan atau belum sehat. Ini juga cara menerapkan
#       realm di lingkungan lokal.
#   bash scripts/keycloak-check.sh ephemeral [rinci]
#       Kontainer Keycloak sekali pakai (image dan konfigurasi server sama dengan compose, DB dev-file),
#       port acak di 127.0.0.1, kredensial admin dan secret klien acak sekali pakai. Untuk CI/verify.
#       Email realm berjalan end-to-end lewat relay ADR-026 sungguhan (F1-T36): Keycloak → STARTTLS ke
#       `email-relay` (rs-backend email-relay, sertifikat dari CA uji yang dibangkitkan saat test, dipercaya
#       lewat conf/truststores Keycloak) → tiruan SES HTTPS yang memvalidasi SigV4 (SesTiruan.java). Kunci
#       akses SES (contoh fiktif AWS) dibaca relay dari OpenBao dev sekali pakai. Tidak ada panggilan ke AWS.
#   "rinci" mencetak setiap baris LULUS; tanpa itu hanya galat dan ringkasan.
#
# Butuh: docker, python3 (pustaka standar saja), cargo dan openssl (mode ephemeral). Kontainer bantu (penangkap
# SMTP pada mode compose; relay, tiruan SES, dan OpenBao pada mode ephemeral) berada di namespace jaringan
# kontainer Keycloak (hanya 127.0.0.1) dan selalu dihapus saat keluar.
# Tidak ada rahasia di berkas ini; nilai rahasia tidak dicetak. Jangan dipakai ke staging/production.

set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "pakai: bash scripts/keycloak-check.sh compose|ephemeral [rinci]" >&2
  exit 2
fi
MODE="$1"
RINCI="${2:-}"
case "$MODE" in compose | ephemeral) ;; *)
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

COMPOSE_FILE="infra/local/compose.yaml"
SINK_PORT=2526
SES_PORT=18443
SINK=""
RELAY=""
BAO=""
KC_EPHEMERAL=""
KERJA="$(mktemp -d)"

bersihkan() {
  local c
  for c in "$RELAY" "$SINK" "$BAO" "$KC_EPHEMERAL"; do
    if [ -n "$c" ]; then docker rm -f "$c" >/dev/null 2>&1 || true; fi
  done
  rm -rf "$KERJA"
}
trap bersihkan EXIT

for alat in docker python3; do
  if ! command -v "$alat" >/dev/null 2>&1; then
    echo "$alat tidak ditemukan di PATH" >&2
    exit 1
  fi
done
for f in "$COMPOSE_FILE" infra/keycloak/keycloak.conf infra/keycloak/realms/pasien.json infra/keycloak/realms/admin.json \
  infra/keycloak/test/keycloak_check.py infra/keycloak/test/kc_http.py infra/keycloak/test/SmtpSink.java \
  infra/keycloak/test/SesTiruan.java \
  infra/keycloak/themes/ruang-sehat/login/theme.properties infra/keycloak/themes/ruang-sehat/login/login.ftl; do
  if [ ! -f "$f" ]; then
    echo "berkas tidak ada: $f" >&2
    exit 1
  fi
done

# Image dipin satu kali di compose.yaml (tag@digest), dipakai juga mode ephemeral.
baca_image() { # baca_image <awalan repo>
  sed -n "s|^[[:space:]]*image:[[:space:]]*\($1:[^[:space:]]*@sha256:[0-9a-f]*\)[[:space:]]*$|\1|p" "$COMPOSE_FILE" | sed -n '1p'
}
IMG_KC="$(baca_image quay.io/keycloak/keycloak)"
IMG_CLI="$(baca_image adorsys/keycloak-config-cli)"
# Mode ephemeral: OpenBao dev untuk kunci SES relay; image postgres (Debian, glibc >= 2.38) hanya sebagai
# lingkungan jalan biner relay hasil `cargo build` (biner butuh glibc 2.38; image Keycloak memuat 2.34).
IMG_BAO="$(baca_image openbao/openbao)"
IMG_PG="$(sed -n 's|^[[:space:]]*image:[[:space:]]*\(postgres:[^[:space:]]*@sha256:[0-9a-f]*\)[[:space:]]*$|\1|p' "$COMPOSE_FILE" | sed -n '1p')"
if [ -z "$IMG_KC" ] || [ -z "$IMG_CLI" ] || [ -z "$IMG_BAO" ] || [ -z "$IMG_PG" ]; then
  echo "image keycloak/keycloak-config-cli/openbao/postgres (tag@digest) tidak terbaca dari $COMPOSE_FILE" >&2
  exit 1
fi

# Nilai dari infra/local/.env (mode compose). Variabel lingkungan yang sudah diset menang.
baca_env() { # baca_env <NAMA> [bawaan]
  local nama="$1" nilai="${!1:-}"
  if [ -z "$nilai" ] && [ -f infra/local/.env ]; then
    nilai="$(sed -n "s/^$nama=//p" infra/local/.env | sed -n '1p')"
  fi
  printf '%s' "${nilai:-${2:-}}"
}

VAR_REALM="RS_KC_SMTP_HOST RS_KC_SMTP_PORT RS_KC_SMTP_FROM RS_KC_PASIEN_MOBILE_REDIRECT_URI \
RS_KC_BACKOFFICE_BFF_REDIRECT_URI RS_KC_BACKOFFICE_BFF_SECRET RS_KC_PORTAL_BFF_REDIRECT_URI RS_KC_PORTAL_BFF_SECRET \
RS_KC_EVENT_READER_PASIEN_SECRET RS_KC_EVENT_READER_ADMIN_SECRET RS_KC_ADMIN_MANAGER_SECRET"

acak() { python3 -c 'import secrets; print(secrets.token_urlsafe(32))'; }

if [ "$MODE" = "compose" ]; then
  COMPOSE=(docker compose -f "$COMPOSE_FILE")
  KC="$("${COMPOSE[@]}" ps -q keycloak 2>/dev/null || true)"
  if [ -z "$KC" ]; then
    echo "GAGAL: Keycloak compose tidak berjalan. Jalankan dulu:" >&2
    echo "  docker compose -f $COMPOSE_FILE up -d --wait" >&2
    exit 1
  fi
  SEHAT="$(docker inspect -f '{{.State.Health.Status}}' "$KC" 2>/dev/null || true)"
  if [ "$SEHAT" != "healthy" ]; then
    echo "GAGAL: Keycloak compose belum sehat (status: ${SEHAT:-tidak diketahui})" >&2
    exit 1
  fi
  export KC_CHECK_BASE_URL="http://127.0.0.1:$(baca_env KEYCLOAK_PORT 18081)"
  KC_CHECK_ADMIN_USER="$(baca_env KEYCLOAK_ADMIN_USER)"
  KC_CHECK_ADMIN_PASSWORD="$(baca_env KEYCLOAK_ADMIN_PASSWORD)"
  export KC_CHECK_ADMIN_USER KC_CHECK_ADMIN_PASSWORD
  for v in $VAR_REALM; do
    nilai="$(baca_env "$v")"
    if [ -z "$nilai" ]; then
      echo "GAGAL: $v belum diisi (infra/local/.env, lihat .env.example)" >&2
      exit 1
    fi
    export "$v=$nilai"
  done
  echo "==> menerapkan realm ke Keycloak compose (keycloak-config-cli)"
  if ! "${COMPOSE[@]}" --profile alat run --rm --no-deps keycloak-config >"$KERJA/config-cli.log" 2>&1; then
    echo "GAGAL: keycloak-config-cli gagal; log terakhir:" >&2
    tail -n 30 "$KERJA/config-cli.log" >&2
    exit 1
  fi
else
  KC_EPHEMERAL="rs-kc-ephemeral-$$-$RANDOM"
  KC="$KC_EPHEMERAL"
  export KC_CHECK_ADMIN_USER="admin-uji"
  KC_CHECK_ADMIN_PASSWORD="$(acak)"
  export KC_CHECK_ADMIN_PASSWORD
  export RS_KC_SMTP_HOST="email-relay" RS_KC_SMTP_PORT="2525" RS_KC_SMTP_FROM="noreply@uji.invalid"
  export RS_KC_PASIEN_MOBILE_REDIRECT_URI="com.example.ruangsehat.pasien:/oauth2redirect"
  export RS_KC_BACKOFFICE_BFF_REDIRECT_URI="http://127.0.0.1:3000/auth/callback"
  export RS_KC_PORTAL_BFF_REDIRECT_URI="http://127.0.0.1:3001/auth/callback"
  RS_KC_BACKOFFICE_BFF_SECRET="$(acak)"
  RS_KC_PORTAL_BFF_SECRET="$(acak)"
  # Klien layanan pengambil event rs-backend worker (F1-T34), satu rahasia per realm.
  RS_KC_EVENT_READER_PASIEN_SECRET="$(acak)"
  RS_KC_EVENT_READER_ADMIN_SECRET="$(acak)"
  # Klien layanan pengelola pengguna admin (F1-T65, US-208).
  RS_KC_ADMIN_MANAGER_SECRET="$(acak)"
  export RS_KC_BACKOFFICE_BFF_SECRET RS_KC_PORTAL_BFF_SECRET RS_KC_EVENT_READER_PASIEN_SECRET RS_KC_EVENT_READER_ADMIN_SECRET
  export RS_KC_ADMIN_MANAGER_SECRET
  for alat in cargo openssl; do
    if ! command -v "$alat" >/dev/null 2>&1; then
      echo "$alat tidak ditemukan di PATH (mode ephemeral: relay email F1-T36)" >&2
      exit 1
    fi
  done
  echo "==> membangun relay (cargo build -p rs-backend)"
  if ! cargo build --quiet --locked -p rs-backend --bin rs-backend; then
    echo "GAGAL: build rs-backend gagal" >&2
    exit 1
  fi
  RELAY_BIN="${CARGO_TARGET_DIR:-$ROOT/target}/debug/rs-backend"
  case "$RELAY_BIN" in /*) ;; *) RELAY_BIN="$ROOT/$RELAY_BIN" ;; esac
  echo "==> CA uji dan sertifikat relay/tiruan SES (openssl, berlaku 1 hari)"
  PKI="$KERJA/pki"
  mkdir -p "$PKI"
  SES_P12_PASSWORD="$(acak)"
  export SES_P12_PASSWORD
  if ! (
    cd "$PKI"
    openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes -keyout ca.key -out ca.pem -days 1 \
      -subj /CN=rs-uji-ca -addext basicConstraints=critical,CA:TRUE -addext keyUsage=critical,keyCertSign,cRLSign
    for nama in relay:DNS:email-relay ses:IP:127.0.0.1; do
      berkas="${nama%%:*}"
      printf 'subjectAltName=%s\nbasicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature\nextendedKeyUsage=serverAuth\n' \
        "${nama#*:}" >"$berkas.ext"
      openssl req -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes -keyout "$berkas.key" -out "$berkas.csr" \
        -subj "/CN=$berkas-uji"
      openssl x509 -req -in "$berkas.csr" -CA ca.pem -CAkey ca.key -CAcreateserial -out "$berkas.pem" -days 1 \
        -extfile "$berkas.ext"
    done
    openssl pkcs12 -export -in ses.pem -inkey ses.key -out ses.p12 -passout env:SES_P12_PASSWORD
    rm -f ca.key ./*.csr
    chmod 644 ca.pem
  ) >"$KERJA/pki.log" 2>&1; then
    echo "GAGAL: pembuatan CA uji gagal; log:" >&2
    cat "$KERJA/pki.log" >&2
    exit 1
  fi
  echo "==> Keycloak ephemeral ($IMG_KC)"
  # Nilai rahasia diteruskan lewat `-e NAMA` (diambil dari lingkungan), tidak muncul di argumen proses.
  # `email-relay` diarahkan ke 127.0.0.1 namespace Keycloak (relay di kontainer yang berbagi namespace itu);
  # CA uji dipercaya lewat conf/truststores, tls-hostname-verifier tetap DEFAULT (KEPUTUSAN-231, SRC-301).
  KC_BOOTSTRAP_ADMIN_USERNAME="$KC_CHECK_ADMIN_USER" KC_BOOTSTRAP_ADMIN_PASSWORD="$KC_CHECK_ADMIN_PASSWORD" \
    docker run -d --name "$KC" -p 127.0.0.1::8080 --add-host email-relay:127.0.0.1 \
    -e KC_BOOTSTRAP_ADMIN_USERNAME -e KC_BOOTSTRAP_ADMIN_PASSWORD \
    -e KC_TRUSTSTORE_PATHS=/opt/keycloak/conf/truststores \
    -v "$ROOT/infra/keycloak/keycloak.conf:/opt/keycloak/conf/keycloak.conf:ro" \
    -v "$ROOT/infra/keycloak/themes/ruang-sehat:/opt/keycloak/themes/ruang-sehat:ro" \
    -v "$PKI/ca.pem:/opt/keycloak/conf/truststores/rs-uji-ca.pem:ro" \
    "$IMG_KC" start-dev >/dev/null
  PORT="$(docker port "$KC" 8080/tcp | sed -n 's/^127\.0\.0\.1:\([0-9]*\)$/\1/p' | sed -n '1p')"
  if [ -z "$PORT" ]; then
    echo "GAGAL: port Keycloak ephemeral tidak terbaca" >&2
    exit 1
  fi
  export KC_CHECK_BASE_URL="http://127.0.0.1:$PORT"
  if ! python3 - "$KC_CHECK_BASE_URL" <<'PY'
import sys, time, urllib.request
batas = time.time() + 240
while time.time() < batas:
    try:
        if urllib.request.urlopen(sys.argv[1] + "/realms/master", timeout=5).status == 200:
            sys.exit(0)
    except Exception:
        pass
    time.sleep(2)
sys.exit(1)
PY
  then
    echo "GAGAL: Keycloak ephemeral tidak siap dalam 240 detik; log terakhir:" >&2
    docker logs --tail 30 "$KC" >&2 || true
    exit 1
  fi
  echo "==> menerapkan realm (keycloak-config-cli $IMG_CLI)"
  # shellcheck disable=SC2086 # daftar nama variabel sengaja dipecah menjadi beberapa -e
  if ! KEYCLOAK_USER="$KC_CHECK_ADMIN_USER" KEYCLOAK_PASSWORD="$KC_CHECK_ADMIN_PASSWORD" \
    docker run --rm --network "container:$KC" \
    -v "$ROOT/infra/keycloak/realms:/config:ro" \
    -e KEYCLOAK_URL=http://127.0.0.1:8080 -e KEYCLOAK_USER -e KEYCLOAK_PASSWORD \
    -e KEYCLOAK_AVAILABILITYCHECK_ENABLED=true -e KEYCLOAK_AVAILABILITYCHECK_TIMEOUT=120s \
    -e 'IMPORT_FILES_LOCATIONS=/config/*.json' -e IMPORT_VARSUBSTITUTION_ENABLED=true -e IMPORT_CACHE_ENABLED=false \
    $(for v in $VAR_REALM; do printf -- '-e %s ' "$v"; done) \
    "$IMG_CLI" >"$KERJA/config-cli.log" 2>&1; then
    echo "GAGAL: keycloak-config-cli gagal; log terakhir:" >&2
    tail -n 30 "$KERJA/config-cli.log" >&2
    exit 1
  fi
fi

tunggu_log() { # tunggu_log <kontainer> <teks> <detik>
  local i
  for i in $(seq 1 "$3"); do
    if docker logs "$1" 2>&1 | grep -q "$2"; then
      return 0
    fi
    if [ "$(docker inspect -f '{{.State.Running}}' "$1" 2>/dev/null || true)" != "true" ]; then
      return 1
    fi
    sleep 1
  done
  return 1
}

if [ "$MODE" = "compose" ]; then
  # Keycloak compose tidak memuat CA uji relay; email diuji lewat penangkap SMTP tanpa TLS (fixture realm
  # sementara di keycloak_check.py). Jalur relay end-to-end hanya di mode ephemeral.
  echo "==> penangkap SMTP uji (namespace jaringan Keycloak, 127.0.0.1:$SINK_PORT)"
  SINK="rs-kc-smtp-sink-$$-$RANDOM"
  docker run -d --name "$SINK" --network "container:$KC" \
    -v "$ROOT/infra/keycloak/test/SmtpSink.java:/sink/SmtpSink.java:ro" \
    --entrypoint java "$IMG_KC" /sink/SmtpSink.java "$SINK_PORT" /tmp/mail >/dev/null
  if ! tunggu_log "$SINK" "smtp-sink siap" 60; then
    echo "GAGAL: penangkap SMTP uji tidak siap" >&2
    docker logs --tail 20 "$SINK" >&2 || true
    exit 1
  fi
  export KC_CHECK_EMAIL_MODE=sink
else
  UJI_UID="$(id -u):$(id -g)"
  # Kunci akses contoh fiktif dari dokumentasi AWS (bukan rahasia); tidak pernah dikirim ke AWS.
  export SES_TIRUAN_KEY_ID="AKIAIOSFODNN7EXAMPLE" SES_TIRUAN_SECRET="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
  echo "==> OpenBao dev sekali pakai (kunci SES relay di KV, namespace jaringan Keycloak)"
  BAO="rs-kc-openbao-$$-$RANDOM"
  BAO_TOKEN="$(acak)"
  export BAO_TOKEN
  BAO_DEV_ROOT_TOKEN_ID="$BAO_TOKEN" docker run -d --name "$BAO" --network "container:$KC" --cap-add IPC_LOCK \
    -e BAO_DEV_ROOT_TOKEN_ID -e BAO_DEV_LISTEN_ADDRESS=127.0.0.1:8200 "$IMG_BAO" server -dev >/dev/null
  siap=""
  for _ in $(seq 1 60); do
    if docker exec -e BAO_ADDR=http://127.0.0.1:8200 "$BAO" bao status >/dev/null 2>&1; then
      siap=1
      break
    fi
    sleep 1
  done
  if [ -z "$siap" ] || ! docker exec -e BAO_ADDR=http://127.0.0.1:8200 -e BAO_TOKEN -e SES_TIRUAN_KEY_ID -e SES_TIRUAN_SECRET \
    "$BAO" sh -c 'bao kv put secret/ruang-sehat/ses/email-relay access_key_id="$SES_TIRUAN_KEY_ID" secret_access_key="$SES_TIRUAN_SECRET"' \
    >/dev/null; then
    echo "GAGAL: OpenBao dev tidak siap atau kunci SES tidak tertulis" >&2
    docker logs --tail 20 "$BAO" >&2 || true
    exit 1
  fi
  echo "==> tiruan SES HTTPS (namespace jaringan Keycloak, 127.0.0.1:$SES_PORT)"
  SINK="rs-kc-ses-tiruan-$$-$RANDOM"
  SES_TIRUAN_P12_PASSWORD="$SES_P12_PASSWORD" docker run -d --name "$SINK" --network "container:$KC" --user "$UJI_UID" \
    -e SES_TIRUAN_P12_PASSWORD -e SES_TIRUAN_KEY_ID -e SES_TIRUAN_SECRET \
    -v "$ROOT/infra/keycloak/test/SesTiruan.java:/t/SesTiruan.java:ro" -v "$PKI:/pki:ro" \
    --entrypoint java "$IMG_KC" /t/SesTiruan.java "$SES_PORT" /pki/ses.p12 /tmp/mail /tmp/ses-json >/dev/null
  if ! tunggu_log "$SINK" "ses-tiruan siap" 60; then
    echo "GAGAL: tiruan SES tidak siap" >&2
    docker logs --tail 20 "$SINK" >&2 || true
    exit 1
  fi
  # Batas per penerima dinaikkan untuk harness: satu user uji menerima banyak email, dan pesan di atas batas
  # dibalas 250 lalu dibuang (ADR-026 butir 4) sehingga kegagalan SES tidak lagi terlihat. Perilaku batas laju
  # diuji di crates/email-relay/tests/relay_smtp.rs.
  echo "==> relay email (rs-backend email-relay, namespace jaringan Keycloak, 127.0.0.1:$RS_KC_SMTP_PORT)"
  RELAY="rs-kc-email-relay-$$-$RANDOM"
  OPENBAO_TOKEN="$BAO_TOKEN" docker run -d --name "$RELAY" --network "container:$KC" --user "$UJI_UID" \
    -v "$RELAY_BIN:/relay/rs-backend:ro" -v "$PKI:/pki:ro" \
    -e LOG_LEVEL=info -e DEPLOY_ENV=local -e EMAIL_RELAY_BIND="127.0.0.1:$RS_KC_SMTP_PORT" \
    -e EMAIL_RELAY_TLS_CERT_FILE=/pki/relay.pem -e EMAIL_RELAY_TLS_KEY_FILE=/pki/relay.key \
    -e EMAIL_RELAY_ALLOWED_FROM="$RS_KC_SMTP_FROM" -e SES_CONFIGURATION_SET=rs-keycloak-uji \
    -e EMAIL_RELAY_PER_RECIPIENT_PER_HOUR=1000 \
    -e SES_LOCAL_ENDPOINT="https://127.0.0.1:$SES_PORT" -e SES_CA_FILE=/pki/ca.pem \
    -e OPENBAO_ADDR=http://127.0.0.1:8200 -e OPENBAO_TOKEN \
    -e SES_CREDENTIALS_KV_PATH=ruang-sehat/ses/email-relay \
    --entrypoint /relay/rs-backend "$IMG_PG" email-relay >/dev/null
  if ! tunggu_log "$RELAY" "relay_listening" 60; then
    echo "GAGAL: relay email tidak siap; log:" >&2
    docker logs --tail 20 "$RELAY" >&2 || true
    exit 1
  fi
  export KC_CHECK_EMAIL_MODE=relay KC_CHECK_RELAY_CONTAINER="$RELAY" KC_CHECK_KC_CONTAINER="$KC" \
    KC_CHECK_SES_CONFIG_SET=rs-keycloak-uji
fi

export KC_CHECK_SINK_CONTAINER="$SINK" KC_CHECK_SINK_PORT="$SINK_PORT"
if [ -n "$RINCI" ]; then export KC_CHECK_RINCI=1; fi
echo "==> test realm pasien dan admin"
python3 -I -B infra/keycloak/test/keycloak_check.py
