#!/usr/bin/env bash
# scripts/ci/pasang-alat.sh: memasang alat pemindai versi persis untuk CI (F1-T06; docs/05-stack.md §3.2).
# Pemakaian: bash scripts/ci/pasang-alat.sh <direktori-tujuan>
#
# - Biner rilis resmi diunduh dari halaman rilis GitHub proyek masing-masing dan diperiksa SHA-256 sebelum
#   diekstrak (ADR-021 butir 4: versi dipin, integritas diperiksa dengan hash). Hash dicocokkan saat ditulis
#   dengan digest aset rilis dari API GitHub dan, bila ada, berkas checksum resmi proyek (SRC-358 s.d. SRC-361).
# - Versi harus sama dengan VERSI_* di scripts/verify.sh; verify.sh menolak versi lain.
# - Hanya Linux x86_64 (runner GitHub-hosted ubuntu-24.04). Mesin pengembang memasang alat sendiri.

set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "pemakaian: $0 <direktori-tujuan>" >&2
  exit 2
fi
TUJUAN="$1"

if [ "$(uname -s)" != "Linux" ] || [ "$(uname -m)" != "x86_64" ]; then
  echo "pasang-alat.sh hanya untuk Linux x86_64 (diberikan: $(uname -s) $(uname -m))" >&2
  exit 1
fi

KERJA="$(mktemp -d)"
trap 'rm -rf "$KERJA"' EXIT
mkdir -p "$TUJUAN"

unduh() { # unduh <url> <sha256> <nama-berkas>: unduh ke $KERJA lalu periksa hash
  curl --proto '=https' --tlsv1.2 -fsSL --retry 3 -o "$KERJA/$3" "$1"
  printf '%s  %s\n' "$2" "$KERJA/$3" | sha256sum -c -
}

# gitleaks 8.30.1 (SRC-358)
unduh "https://github.com/gitleaks/gitleaks/releases/download/v8.30.1/gitleaks_8.30.1_linux_x64.tar.gz" \
  "551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb" "gitleaks.tar.gz"
mkdir -p "$KERJA/gitleaks"
tar -xzf "$KERJA/gitleaks.tar.gz" -C "$KERJA/gitleaks" gitleaks
install -m 0755 "$KERJA/gitleaks/gitleaks" "$TUJUAN/gitleaks"

# osv-scanner 2.7.1 (SRC-359): biner tunggal
unduh "https://github.com/google/osv-scanner/releases/download/v2.7.1/osv-scanner_linux_amd64" \
  "fb7dab19a8bd9bbe5c3189f09d4b7ee1820c9553a23bac202648a56efd2f6ee8" "osv-scanner"
install -m 0755 "$KERJA/osv-scanner" "$TUJUAN/osv-scanner"

# cargo-deny 0.21.0 (SRC-360)
unduh "https://github.com/EmbarkStudios/cargo-deny/releases/download/0.21.0/cargo-deny-0.21.0-x86_64-unknown-linux-musl.tar.gz" \
  "a2170b854ff697e79e39c4c538981c9c1bdd803addbae6719a4cb66b4585a903" "cargo-deny.tar.gz"
tar -xzf "$KERJA/cargo-deny.tar.gz" -C "$KERJA" cargo-deny-0.21.0-x86_64-unknown-linux-musl/cargo-deny
install -m 0755 "$KERJA/cargo-deny-0.21.0-x86_64-unknown-linux-musl/cargo-deny" "$TUJUAN/cargo-deny"

# cargo-audit 0.23.0 (SRC-361)
unduh "https://github.com/rustsec/rustsec/releases/download/cargo-audit/v0.23.0/cargo-audit-x86_64-unknown-linux-musl-v0.23.0.tgz" \
  "4f01c74ed84b2dffd0b1d37012a62cdf2a92c96e25bdf8eec7fcc0d4d3eb7811" "cargo-audit.tgz"
tar -xzf "$KERJA/cargo-audit.tgz" -C "$KERJA" cargo-audit-x86_64-unknown-linux-musl-v0.23.0/cargo-audit
install -m 0755 "$KERJA/cargo-audit-x86_64-unknown-linux-musl-v0.23.0/cargo-audit" "$TUJUAN/cargo-audit"

echo "Alat terpasang di $TUJUAN:"
"$TUJUAN/gitleaks" version
"$TUJUAN/osv-scanner" --version | sed -n '1p'
"$TUJUAN/cargo-deny" --version
"$TUJUAN/cargo-audit" --version
