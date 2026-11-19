#!/usr/bin/env bash
# scripts/ci/periksa-lingkungan-bersih.sh: memastikan tidak ada konfigurasi di luar repo yang dapat mengubah
# hasil scripts/verify.sh di CI (F1-T06; catatan review keamanan F1-T05: konfigurasi cargo level user).
#
# verify.sh sudah menolak variabel lingkungan berbahaya dan .cargo/config(.toml) di dalam repo. Yang tidak
# dapat diperiksanya tanpa melarang konfigurasi sah di mesin pengembang adalah berkas di luar repo:
# - $CARGO_HOME/config(.toml): dapat mengganti rustflags, runner, alias, atau registry cargo;
# - .cargo/config(.toml) di direktori induk repo: dibaca cargo secara hierarkis;
# - $CARGO_HOME/audit.toml: dapat mengabaikan advisory atau menaikkan ambang cargo-audit (SRC-361).
# Di runner GitHub-hosted yang bersih berkas ini tidak ada (SRC-357); skrip ini membuktikannya setiap jalan.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CARGO_HOME_EFEKTIF="${CARGO_HOME:-$HOME/.cargo}"
TEMUAN=0

laporkan() {
  echo "LINGKUNGAN TIDAK BERSIH: $1" >&2
  TEMUAN=$((TEMUAN + 1))
}

for berkas in config config.toml audit.toml; do
  if [ -e "$CARGO_HOME_EFEKTIF/$berkas" ]; then
    laporkan "$CARGO_HOME_EFEKTIF/$berkas ada"
  fi
done

# Direktori induk repo sampai akar (repo itu sendiri diperiksa verify.sh).
dir="$(dirname "$ROOT")"
while :; do
  for berkas in config config.toml; do
    if [ -e "$dir/.cargo/$berkas" ]; then
      laporkan "$dir/.cargo/$berkas ada"
    fi
  done
  [ "$dir" = "/" ] && break
  dir="$(dirname "$dir")"
done

if [ "$TEMUAN" -ne 0 ]; then
  echo "periksa-lingkungan-bersih: $TEMUAN temuan; verifikasi CI tidak dapat dipercaya." >&2
  exit 1
fi
echo "periksa-lingkungan-bersih: tidak ada konfigurasi cargo/cargo-audit di luar repo (CARGO_HOME=$CARGO_HOME_EFEKTIF)."
