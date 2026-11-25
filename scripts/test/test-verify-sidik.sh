#!/usr/bin/env bash
# scripts/test/test-verify-sidik.sh: test unit cepat (detik, tanpa Docker/jaringan) untuk penyaringan kelas
# masukan sidik cache scripts/verify.sh (KEPUTUSAN-301; review T1 B-1).
#
# Fungsi diambil apa adanya dari scripts/verify.sh (definisi "nama() {" sampai "}" di kolom pertama) lalu
# dijalankan terhadap manifest sintetis di direktori sementara. Regresi B-1: bila tidak ada rujukan docs/,
# penyaringan kelas rust/web tidak boleh kosong; penjaga saring_kelas_aman harus gagal-tertutup.
# Dijalankan oleh CI dan oleh scripts/test/test-verify.sh. Portabel bash 3.2. Tidak menerima argumen.

set -euo pipefail

if [ "$#" -ne 0 ]; then
  echo "test-verify-sidik.sh: tidak menerima argumen." >&2
  exit 2
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VERIFY="$ROOT/scripts/verify.sh"
KERJA="$(mktemp -d "${TMPDIR:-/tmp}/test-verify-sidik.XXXXXX")"
trap 'rm -rf "$KERJA"' EXIT

ambil_fungsi() { # ambil_fungsi <nama>: cetak definisi fungsi dari verify.sh; gagal bila tidak ada
  local teks
  teks="$(awk -v n="$1" '$0 ~ "^" n "\\(\\) \\{" { p = 1 } p { print } p && /^}$/ { exit }' "$VERIFY")"
  if [ -z "$teks" ]; then
    echo "fungsi $1 tidak ditemukan di scripts/verify.sh" >&2
    return 1
  fi
  printf '%s\n' "$teks"
}

for f in sha256_alir docs_dirujuk saring_kelas saring_kelas_aman; do
  eval "$(ambil_fungsi "$f")"
done

LULUS=0
GAGAL=0
cek() { # cek <nama> <perintah...>: perintah harus berhasil
  local nama="$1"
  shift
  if "$@"; then
    LULUS=$((LULUS + 1))
    echo "LULUS  $nama"
  else
    GAGAL=$((GAGAL + 1))
    echo "GAGAL  $nama" >&2
  fi
}
ada() { LC_ALL=C grep -q "^$1$(printf '\t')" "$2"; }
tidak_ada() { ! ada "$1" "$2"; }
gagal_tertutup() { ! saring_kelas_aman "$1" "$CACHE_TMP/aman"; }

# Pohon sintetis: tidak ada berkas sumber yang menyebut docs/.
REPO="$KERJA/repo"
mkdir -p "$REPO/crates/a/src" "$REPO/web/app" "$REPO/docs" "$REPO/scripts"
printf 'pub fn a() {}\n' >"$REPO/crates/a/src/lib.rs"
printf 'export const a = 1;\n' >"$REPO/web/app/a.ts"
printf '# dokumen\n' >"$REPO/docs/x.md"
printf '# dokumen lain\n' >"$REPO/docs/y.md"
printf '#!/usr/bin/env bash\n' >"$REPO/scripts/verify.sh"
cd "$REPO"

CACHE_TMP="$KERJA/cache"
mkdir -p "$CACHE_TMP"
tulis_manifest() { # dari berkas di pohon sintetis, bentuk sama dengan manifest_bangun ("path<TAB>c<TAB>hash")
  local f
  for f in crates/a/src/lib.rs docs/x.md docs/y.md scripts/verify.sh web/app/a.ts; do
    printf '%s\tc\t%s\n' "$f" "$(sha256_alir <"$f" | cut -c1-64)"
  done | LC_ALL=C sort >"$CACHE_TMP/manifest"
}
tulis_manifest

# 1. Rujukan docs kosong, kelas rust (B-1): berkas kode dan skrip tetap masuk.
saring_kelas rust >"$CACHE_TMP/rust.1"
cek "rust-tanpa-rujukan: berkas .rs masuk" ada crates/a/src/lib.rs "$CACHE_TMP/rust.1"
cek "rust-tanpa-rujukan: scripts/verify.sh masuk" ada scripts/verify.sh "$CACHE_TMP/rust.1"
cek "rust-tanpa-rujukan: web/ tidak masuk" tidak_ada web/app/a.ts "$CACHE_TMP/rust.1"
cek "rust-tanpa-rujukan: docs/ tak dirujuk tidak masuk" tidak_ada docs/x.md "$CACHE_TMP/rust.1"
cek "rust-tanpa-rujukan: saring_kelas_aman lulus" saring_kelas_aman rust "$CACHE_TMP/aman"

# 2. Rujukan docs kosong, kelas web (B-1).
saring_kelas web >"$CACHE_TMP/web.1"
cek "web-tanpa-rujukan: berkas web masuk" ada web/app/a.ts "$CACHE_TMP/web.1"
cek "web-tanpa-rujukan: scripts/verify.sh masuk" ada scripts/verify.sh "$CACHE_TMP/web.1"
cek "web-tanpa-rujukan: crates/ tidak masuk" tidak_ada crates/a/src/lib.rs "$CACHE_TMP/web.1"
cek "web-tanpa-rujukan: saring_kelas_aman lulus" saring_kelas_aman web "$CACHE_TMP/aman"

# 3. Perubahan satu berkas kode tanpa rujukan docs mengubah keluaran kelas (sidik pasti berbeda).
printf 'pub fn a() { let _ = 1; }\n' >crates/a/src/lib.rs
printf 'export const a = 2;\n' >web/app/a.ts
tulis_manifest
saring_kelas rust >"$CACHE_TMP/rust.2"
saring_kelas web >"$CACHE_TMP/web.2"
cek "rust-tanpa-rujukan: ubah .rs mengubah kelas" sh -c '! cmp -s "$0" "$1"' "$CACHE_TMP/rust.1" "$CACHE_TMP/rust.2"
cek "web-tanpa-rujukan: ubah .ts mengubah kelas" sh -c '! cmp -s "$0" "$1"' "$CACHE_TMP/web.1" "$CACHE_TMP/web.2"

# 4. Rujukan bernama: hanya docs/x.md yang masuk kelas rust.
printf 'const P: &str = "docs/x.md";\n' >crates/a/src/lib.rs
tulis_manifest
saring_kelas rust >"$CACHE_TMP/rust.3"
cek "rust-rujukan-bernama: docs/x.md masuk" ada docs/x.md "$CACHE_TMP/rust.3"
cek "rust-rujukan-bernama: docs/y.md tidak masuk" tidak_ada docs/y.md "$CACHE_TMP/rust.3"
cek "rust-rujukan-bernama: .rs tetap masuk" ada crates/a/src/lib.rs "$CACHE_TMP/rust.3"

# 5. Rujukan dinamis ("docs{" dan "docs(", review N-3): seluruh docs/ masuk.
printf 'const P: &str = concat!("docs{", "}");\n' >crates/a/src/lib.rs
tulis_manifest
saring_kelas rust >"$CACHE_TMP/rust.4"
cek "rust-rujukan-dinamis-kurawal: docs/y.md masuk" ada docs/y.md "$CACHE_TMP/rust.4"
printf 'export const p = docs(1);\n' >web/app/a.ts
tulis_manifest
saring_kelas web >"$CACHE_TMP/web.4"
cek "web-rujukan-dinamis-kurung: docs/x.md masuk" ada docs/x.md "$CACHE_TMP/web.4"
cek "web-rujukan-dinamis-kurung: docs/y.md masuk" ada docs/y.md "$CACHE_TMP/web.4"

# 6. Penjaga gagal-tertutup: manifest kosong, tanpa scripts/verify.sh, atau kelas tidak dikenal.
: >"$CACHE_TMP/manifest"
cek "aman: manifest kosong ditolak (rust)" gagal_tertutup rust
cek "aman: manifest kosong ditolak (semua)" gagal_tertutup semua
tulis_manifest
LC_ALL=C grep -v '^scripts/verify\.sh' "$CACHE_TMP/manifest" >"$CACHE_TMP/m" && mv "$CACHE_TMP/m" "$CACHE_TMP/manifest"
cek "aman: tanpa scripts/verify.sh ditolak (web)" gagal_tertutup web
cek "aman: tanpa scripts/verify.sh ditolak (semua-git)" gagal_tertutup semua-git
tulis_manifest
cek "aman: kelas tidak dikenal ditolak" gagal_tertutup lain

echo ""
echo "Lulus: $LULUS, gagal: $GAGAL"
if [ "$GAGAL" -ne 0 ] || [ "$LULUS" -eq 0 ]; then
  echo "HASIL: MERAH"
  exit 1
fi
echo "HASIL: HIJAU"
