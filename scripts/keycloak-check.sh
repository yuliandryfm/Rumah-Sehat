#!/usr/bin/env bash
# scripts/keycloak-check.sh: menerapkan realm Keycloak sebagai kode (infra/keycloak/realms/*.json) dengan
# keycloak-config-cli, lalu menjalankan test konfigurasi dan perilaku ke Keycloak sungguhan (F1-T31).
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
#   "rinci" mencetak setiap baris LULUS; tanpa itu hanya galat dan ringkasan.
#
# Butuh: docker, python3 (pustaka standar saja). Penangkap SMTP uji dijalankan di kontainer image Keycloak
# yang sama, di namespace jaringan kontainer Keycloak (hanya 127.0.0.1), dan selalu dihapus saat keluar.
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
SINK=""
KC_EPHEMERAL=""
KERJA="$(mktemp -d)"

bersihkan() {
  if [ -n "$SINK" ]; then docker rm -f "$SINK" >/dev/null 2>&1 || true; fi
  if [ -n "$KC_EPHEMERAL" ]; then docker rm -f "$KC_EPHEMERAL" >/dev/null 2>&1 || true; fi
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
  infra/keycloak/test/keycloak_check.py infra/keycloak/test/kc_http.py infra/keycloak/test/SmtpSink.java; do
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
if [ -z "$IMG_KC" ] || [ -z "$IMG_CLI" ]; then
  echo "image keycloak/keycloak-config-cli (tag@digest) tidak terbaca dari $COMPOSE_FILE" >&2
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
RS_KC_BACKOFFICE_BFF_REDIRECT_URI RS_KC_BACKOFFICE_BFF_SECRET RS_KC_PORTAL_BFF_REDIRECT_URI RS_KC_PORTAL_BFF_SECRET"

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
  export RS_KC_BACKOFFICE_BFF_SECRET RS_KC_PORTAL_BFF_SECRET
  echo "==> Keycloak ephemeral ($IMG_KC)"
  # Nilai rahasia diteruskan lewat `-e NAMA` (diambil dari lingkungan), tidak muncul di argumen proses.
  KC_BOOTSTRAP_ADMIN_USERNAME="$KC_CHECK_ADMIN_USER" KC_BOOTSTRAP_ADMIN_PASSWORD="$KC_CHECK_ADMIN_PASSWORD" \
    docker run -d --name "$KC" -p 127.0.0.1::8080 \
    -e KC_BOOTSTRAP_ADMIN_USERNAME -e KC_BOOTSTRAP_ADMIN_PASSWORD \
    -v "$ROOT/infra/keycloak/keycloak.conf:/opt/keycloak/conf/keycloak.conf:ro" \
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

echo "==> penangkap SMTP uji (namespace jaringan Keycloak, 127.0.0.1:$SINK_PORT)"
SINK="rs-kc-smtp-sink-$$-$RANDOM"
docker run -d --name "$SINK" --network "container:$KC" \
  -v "$ROOT/infra/keycloak/test/SmtpSink.java:/sink/SmtpSink.java:ro" \
  --entrypoint java "$IMG_KC" /sink/SmtpSink.java "$SINK_PORT" /tmp/mail >/dev/null
siap=""
for _ in $(seq 1 60); do
  if docker logs "$SINK" 2>/dev/null | grep -q "smtp-sink siap"; then
    siap=1
    break
  fi
  sleep 1
done
if [ -z "$siap" ]; then
  echo "GAGAL: penangkap SMTP uji tidak siap" >&2
  docker logs --tail 20 "$SINK" >&2 || true
  exit 1
fi

export KC_CHECK_SINK_CONTAINER="$SINK" KC_CHECK_SINK_PORT="$SINK_PORT"
if [ -n "$RINCI" ]; then export KC_CHECK_RINCI=1; fi
echo "==> test realm pasien dan admin"
python3 -I -B infra/keycloak/test/keycloak_check.py
