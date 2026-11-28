#!/usr/bin/env bash
# scripts/ci/pasang-qpdf.sh: memasang qpdf 12.4.2 dari rilis resmi GitHub (KEPUTUSAN-451, TD-48, TD-64).
# Pemakaian: bash scripts/ci/pasang-qpdf.sh [direktori-tujuan]
#   Bawaan: $HOME/.local/share/ruang-sehat/qpdf-12.4.2 (lokasi yang dicari test corporate_documents_pdf_qpdf.rs).
#   Biner di <tujuan>/bin/qpdf, pustaka bawaan rilis di <tujuan>/lib.
#
# Skrip ini sumber tunggal pemasangan qpdf: CI (job test-qpdf) memakainya, dan image api wajib memakainya
# (versi dan hash yang sama dengan yang diuji; TD-48).
#
# - Aset: qpdf-12.4.2-bin-linux-x86_64.zip dari https://github.com/qpdf/qpdf/releases/tag/v12.4.2 (SRC-862).
# - sha256 dicocokkan dari tiga sumber saat ditulis: berkas checksum resmi rilis (qpdf-12.4.2.sha256), digest
#   aset di API GitHub, dan hash berkas unduhan (SRC-862, SRC-981).
# - Gagal-tertutup: hash tidak cocok, unduhan gagal, atau versi biner bukan 12.4.2 = keluar non-nol dan
#   direktori tujuan tidak dibuat/diubah (ekstraksi ke direktori sementara, dipindah hanya setelah semua lolos).
# - Tanpa sudo, tanpa masukan luar kecuali argumen direktori tujuan.
# - Hanya Linux x86_64.

set -euo pipefail

VERSI="12.4.2"
URL="https://github.com/qpdf/qpdf/releases/download/v${VERSI}/qpdf-${VERSI}-bin-linux-x86_64.zip"
SHA256="db367d897829f22c4198ce1094143c9d467bd6ee7dfabc44ba6f02056b24f8b1"

if [ "$#" -gt 1 ]; then
  echo "pemakaian: $0 [direktori-tujuan]" >&2
  exit 2
fi
TUJUAN="${1:-$HOME/.local/share/ruang-sehat/qpdf-$VERSI}"

if [ "$(uname -s)" != "Linux" ] || [ "$(uname -m)" != "x86_64" ]; then
  echo "pasang-qpdf.sh hanya untuk Linux x86_64 (diberikan: $(uname -s) $(uname -m))" >&2
  exit 1
fi
for alat in curl unzip sha256sum; do
  command -v "$alat" >/dev/null 2>&1 || { echo "pasang-qpdf: $alat tidak ditemukan" >&2; exit 1; }
done

KERJA="$(mktemp -d)"
trap 'rm -rf "$KERJA"' EXIT

curl --proto '=https' --tlsv1.2 -fsSL --retry 3 -o "$KERJA/qpdf.zip" "$URL"
printf '%s  %s\n' "$SHA256" "$KERJA/qpdf.zip" | sha256sum -c - >&2 || {
  echo "pasang-qpdf: sha256 TIDAK cocok, tidak ada yang dipasang" >&2
  exit 1
}

mkdir "$KERJA/isi"
unzip -q "$KERJA/qpdf.zip" -d "$KERJA/isi"
test -f "$KERJA/isi/bin/qpdf"
chmod 0755 "$KERJA/isi/bin/"*
# Lingkungan dikosongkan seperti saat dijalankan backend (pdf.rs: env_clear).
versi="$(env -i "$KERJA/isi/bin/qpdf" --version | sed -n '1p')"
if [ "$versi" != "qpdf version $VERSI" ]; then
  echo "pasang-qpdf: versi biner tidak sesuai (diharapkan 'qpdf version $VERSI', dapat '$versi')" >&2
  exit 1
fi

if [ -e "$TUJUAN" ]; then
  echo "pasang-qpdf: $TUJUAN sudah ada; hapus dulu bila ingin memasang ulang" >&2
  exit 1
fi
mkdir -p "$(dirname "$TUJUAN")"
mv "$KERJA/isi" "$TUJUAN"
echo "qpdf $VERSI terpasang di $TUJUAN/bin/qpdf"
