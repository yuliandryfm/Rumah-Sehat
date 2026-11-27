#!/usr/bin/env bash
# scripts/test/test-verify-kelompok.sh: test kelengkapan kelompok fixture scripts/test/test-verify.sh (KEPUTUSAN-340).
#
# Memastikan tidak ada fixture yang diam-diam tidak pernah jalan di CI:
#   1. setiap fungsi blok_<nama> di test-verify.sh (daftar dari bash lewat --daftar-fungsi, bukan regex) tercantum tepat satu kali di kelompok mana pun
#      (gabungan semua kelompok = seluruh blok; fixture baru yang lupa dimasukkan membuat test ini gagal);
#   2. setiap blok yang tercantum punya fungsinya; setiap kelompok di KELOMPOK_URUTAN punya isi;
#   3. tidak ada pemanggilan fixture_baru di luar fungsi blok_ (fixture liar di tingkat atas tidak masuk kelompok);
#   4. daftar kelompok di matrix .github/workflows/ci.yml sama persis dengan KELOMPOK_URUTAN;
#   5. pilihan kelompok yang tidak dikenal gagal (kode 2), bukan menjalankan 0 fixture; argumen lain ditolak.
# Pemeriksa itu sendiri diuji terhadap salinan yang sengaja dirusak (tes negatif). Cepat (detik), tanpa Docker
# dan tanpa jaringan; dijalankan CI di job verify dan boleh dijalankan lokal. Portabel bash 3.2.
# Tidak menerima argumen.

set -euo pipefail

if [ "$#" -ne 0 ]; then
  echo "test-verify-kelompok.sh: tidak menerima argumen." >&2
  exit 2
fi

for v in $(env | sed -n 's/^\(GIT_[A-Za-z0-9_]*\)=.*/\1/p'); do
  unset "$v"
done
unset RS_TEST_VERIFY_KELOMPOK

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KERJA="$(mktemp -d "${TMPDIR:-/tmp}/test-verify-kelompok.XXXXXX")"
trap 'rm -rf "$KERJA"' EXIT

LULUS=0
GAGAL=0

# periksa_skrip <test-verify.sh>: kode 0 bila kelompok lengkap; bila tidak, cetak alasan ke stderr dan kode 1.
periksa_skrip() {
  local skrip="$1" d="$KERJA/periksa.$$" ok=0 pasangan fungsi terdaftar urutan label
  rm -rf "$d"
  mkdir -p "$d/scripts/test"
  cp "$skrip" "$d/scripts/test/test-verify.sh"
  pasangan="$(bash "$d/scripts/test/test-verify.sh" --daftar)" || { echo "  --daftar gagal" >&2; return 1; }
  # Daftar fungsi = gabungan dua sumber (R-1, KEPUTUSAN-394):
  #   (a) bash sendiri (`--daftar-fungsi`, declare -F): gaya penulisan apa pun terhitung, tetapi hanya fungsi yang
  #       benar-benar terdefinisi sebelum alur utama;
  #   (b) pindaian teks semua gaya (blok_x() {, blok_x () {, function blok_x {, function blok_x() {): menangkap fungsi
  #       yang ditaruh setelah alur utama atau exit, yang tidak pernah terdefinisi sehingga fixture-nya tidak pernah jalan.
  # Fungsi yang ada di teks tetapi tidak di (a) membuat pemeriksa GAGAL.
  fungsi_bash="$(bash "$d/scripts/test/test-verify.sh" --daftar-fungsi)" || { echo "  --daftar-fungsi gagal" >&2; return 1; }
  fungsi_teks="$(sed -nE 's/^[[:space:]]*(function[[:space:]]+)?(blok_[a-z0-9_]+)[[:space:]]*(\(\))?[[:space:]]*\{.*/\2/p' "$skrip" | sort -u)"
  tersembunyi="$(comm -13 <(printf '%s\n' "$fungsi_bash" | sort -u) <(printf '%s\n' "$fungsi_teks"))"
  if [ -n "$tersembunyi" ]; then
    echo "  fungsi blok_ didefinisikan setelah alur utama (tidak pernah terdefinisi, fixture tidak jalan): $(printf '%s\n' "$tersembunyi" | tr '\n' ' ')" >&2
    ok=1
  fi
  fungsi="$(printf '%s\n%s\n' "$fungsi_bash" "$fungsi_teks" | sed '/^$/d' | sort -u)"
  terdaftar="$(printf '%s\n' "$pasangan" | awk 'NF == 2 { print "blok_" $2 }' | sort)"
  if [ -z "$fungsi" ]; then
    echo "  tidak ada fungsi blok_ di $skrip" >&2
    ok=1
  fi
  if [ "$fungsi" != "$(printf '%s\n' "$terdaftar" | sort -u)" ]; then
    echo "  gabungan kelompok tidak sama dengan seluruh blok. Beda (< fungsi tanpa kelompok, > kelompok tanpa fungsi):" >&2
    diff <(printf '%s\n' "$fungsi") <(printf '%s\n' "$terdaftar" | sort -u) >&2 || true
    ok=1
  fi
  if [ -n "$(printf '%s\n' "$terdaftar" | uniq -d)" ]; then
    echo "  blok tercantum lebih dari satu kali: $(printf '%s\n' "$terdaftar" | uniq -d | tr '\n' ' ')" >&2
    ok=1
  fi
  urutan="$(sed -n 's/^KELOMPOK_URUTAN="\(.*\)"$/\1/p' "$skrip" | tr ' ' '\n' | sort)"
  label="$(sed -n 's/^    \([a-z0-9-]*\)) echo ".*"  *;;$/\1/p' "$skrip" | sort)"
  if [ -z "$urutan" ] || [ "$urutan" != "$label" ]; then
    echo "  KELOMPOK_URUTAN tidak sama dengan label di kelompok_blok" >&2
    ok=1
  fi
  if [ "$urutan" != "$(printf '%s\n' "$pasangan" | awk 'NF == 2 { print $1 }' | sort -u)" ]; then
    echo "  ada kelompok tanpa isi (atau isi tanpa kelompok)" >&2
    ok=1
  fi
  # Fixture liar: pemanggilan fixture_baru di kolom pertama (di luar fungsi) tidak masuk kelompok mana pun.
  if grep -Eq '^[A-Za-z_]+=.*fixture_baru' "$skrip"; then
    echo "  fixture_baru dipanggil di luar fungsi blok_ (tidak masuk kelompok)" >&2
    ok=1
  fi
  return "$ok"
}

# periksa_matrix <ci.yml> <test-verify.sh>: daftar kelompok matrix = KELOMPOK_URUTAN.
periksa_matrix() {
  local ci="$1" skrip="$2" di_ci urutan
  di_ci="$(awk '
    /^ +kelompok:[[:space:]]*$/ { dalam = 1; next }
    dalam && /^ +- [a-z0-9-]+[[:space:]]*$/ { sub(/^ +- /, ""); sub(/[[:space:]]+$/, ""); print; next }
    dalam { dalam = 0 }
  ' "$ci" | sort)"
  urutan="$(sed -n 's/^KELOMPOK_URUTAN="\(.*\)"$/\1/p' "$skrip" | tr ' ' '\n' | sort)"
  if [ -z "$di_ci" ]; then
    echo "  matrix kelompok tidak ditemukan di $ci" >&2
    return 1
  fi
  if [ "$di_ci" != "$urutan" ]; then
    echo "  matrix ci.yml tidak sama dengan KELOMPOK_URUTAN. Beda (< ci.yml, > test-verify.sh):" >&2
    diff <(printf '%s\n' "$di_ci") <(printf '%s\n' "$urutan") >&2 || true
    return 1
  fi
  if [ "$(printf '%s\n' "$di_ci" | uniq -d)" != "" ]; then
    echo "  kelompok ganda di matrix" >&2
    return 1
  fi
}

hasil() { # hasil <nama> <kode-aktual> <kode-diharapkan>
  if [ "$2" = "$3" ]; then
    LULUS=$((LULUS + 1))
    echo "LULUS  $1"
  else
    GAGAL=$((GAGAL + 1))
    echo "GAGAL  $1 (kode $2, diharapkan $3)" >&2
  fi
}

ASLI="$ROOT/scripts/test/test-verify.sh"
CI_ASLI="$ROOT/.github/workflows/ci.yml"

# sisip_fungsi <teks-printf> <keluaran>: salinan test-verify.sh dengan fungsi tambahan disisipkan SEBELUM baris
# KELOMPOK_URUTAN (fungsi di bawah alur utama tidak pernah terdefinisi, jadi tidak mewakili fixture yang hilang).
sisip_fungsi() {
  local teks="$1" keluaran="$2"
  awk -v teks="$teks" '/^KELOMPOK_URUTAN=/ { printf "%s", teks } { print }' "$ASLI" >"$keluaran"
}

# --- Positif: repo apa adanya ---
k=0
periksa_skrip "$ASLI" || k=$?
hasil "kelompok-lengkap-di-repo" "$k" 0
k=0
periksa_matrix "$CI_ASLI" "$ASLI" || k=$?
hasil "matrix-ci-sama-dengan-kelompok" "$k" 0

# --- Negatif: pemeriksa harus gagal pada salinan yang dirusak ---
# Fixture baru yang lupa dimasukkan ke kelompok.
sisip_fungsi "blok_lupa_didaftarkan() {\n  :\n}\n" "$KERJA/lupa.sh"
k=0
periksa_skrip "$KERJA/lupa.sh" 2>"$KERJA/lupa.err" || k=$?
hasil "negatif-blok-lupa-didaftarkan-gagal" "$k" 1
grep -q 'blok_lupa_didaftarkan' "$KERJA/lupa.err" && hasil "negatif-blok-lupa-pesan-menyebut-blok" 0 0 || hasil "negatif-blok-lupa-pesan-menyebut-blok" 1 0

# Fixture baru dengan gaya penulisan fungsi lain (regex lama melewatkan keduanya).
sisip_fungsi "blok_gaya_spasi () {\n  :\n}\n" "$KERJA/gaya1.sh"
k=0
periksa_skrip "$KERJA/gaya1.sh" 2>"$KERJA/gaya1.err" || k=$?
hasil "negatif-blok-gaya-spasi-gagal" "$k" 1
grep -q 'blok_gaya_spasi' "$KERJA/gaya1.err" && hasil "negatif-blok-gaya-spasi-pesan" 0 0 || hasil "negatif-blok-gaya-spasi-pesan" 1 0
sisip_fungsi "function blok_gaya_function {\n  :\n}\n" "$KERJA/gaya2.sh"
k=0
periksa_skrip "$KERJA/gaya2.sh" 2>"$KERJA/gaya2.err" || k=$?
hasil "negatif-blok-gaya-function-gagal" "$k" 1
grep -q 'blok_gaya_function' "$KERJA/gaya2.err" && hasil "negatif-blok-gaya-function-pesan" 0 0 || hasil "negatif-blok-gaya-function-pesan" 1 0

# R-1: blok di akhir berkas (setelah alur utama dan exit) tidak pernah terdefinisi; harus GAGAL walau terdaftar di kelompok.
{ cat "$ASLI"; printf 'blok_di_akhir() {\n  :\n}\n'; } >"$KERJA/akhir.sh"
k=0
periksa_skrip "$KERJA/akhir.sh" 2>"$KERJA/akhir.err" || k=$?
hasil "negatif-blok-di-akhir-berkas-gagal" "$k" 1
grep -q 'setelah alur utama' "$KERJA/akhir.err" && grep -q 'blok_di_akhir' "$KERJA/akhir.err" && hasil "negatif-blok-di-akhir-pesan" 0 0 || hasil "negatif-blok-di-akhir-pesan" 1 0
{ cat "$ASLI"; printf 'function blok_di_akhir_f {\n  :\n}\n'; } >"$KERJA/akhir2.sh"
k=0
periksa_skrip "$KERJA/akhir2.sh" 2>"$KERJA/akhir2.err" || k=$?
hasil "negatif-blok-function-di-akhir-gagal" "$k" 1

# Blok dicabut dari kelompok.
sed 's/^    web-a) echo "web_artefak web_lock_basi" ;;$/    web-a) echo "web_artefak" ;;/' "$ASLI" >"$KERJA/dicabut.sh"
k=0
periksa_skrip "$KERJA/dicabut.sh" 2>/dev/null || k=$?
hasil "negatif-blok-dicabut-dari-kelompok-gagal" "$k" 1

# Blok tercantum di dua kelompok.
sed 's/^    web-a) echo "web_artefak web_lock_basi" ;;$/    web-a) echo "web_artefak web_lock_basi fmt" ;;/' "$ASLI" >"$KERJA/ganda.sh"
k=0
periksa_skrip "$KERJA/ganda.sh" 2>/dev/null || k=$?
hasil "negatif-blok-ganda-gagal" "$k" 1

# Kelompok mereferensikan blok yang tidak ada fungsinya.
sed 's/^    web-a) echo "web_artefak web_lock_basi" ;;$/    web-a) echo "web_artefak web_lock_basi hantu" ;;/' "$ASLI" >"$KERJA/hantu.sh"
k=0
periksa_skrip "$KERJA/hantu.sh" 2>/dev/null || k=$?
hasil "negatif-blok-tanpa-fungsi-gagal" "$k" 1

# Fixture liar di tingkat atas.
{ cat "$ASLI"; printf 'r="$(fixture_baru liar)"\n'; } >"$KERJA/liar.sh"
k=0
periksa_skrip "$KERJA/liar.sh" 2>/dev/null || k=$?
hasil "negatif-fixture-liar-tingkat-atas-gagal" "$k" 1

# Kelompok di ci.yml hilang, atau ada kelompok asing.
grep -v '^ *- web-a *$' "$CI_ASLI" >"$KERJA/ci-kurang.yml"
k=0
periksa_matrix "$KERJA/ci-kurang.yml" "$ASLI" 2>/dev/null || k=$?
hasil "negatif-matrix-kurang-kelompok-gagal" "$k" 1
awk '{ print } /^ +- web-a[[:space:]]*$/ { sub(/web-a/, "asing"); print }' "$CI_ASLI" >"$KERJA/ci-lebih.yml"
k=0
periksa_matrix "$KERJA/ci-lebih.yml" "$ASLI" 2>/dev/null || k=$?
hasil "negatif-matrix-kelompok-asing-gagal" "$k" 1

# --- Pilihan kelompok ---
k=0
RS_TEST_VERIFY_KELOMPOK=tidak-ada bash "$ASLI" >"$KERJA/pilih.out" 2>"$KERJA/pilih.err" || k=$?
hasil "kelompok-tidak-dikenal-gagal-kode-2" "$k" 2
grep -q 'kelompok tidak dikenal' "$KERJA/pilih.err" && hasil "kelompok-tidak-dikenal-pesan" 0 0 || hasil "kelompok-tidak-dikenal-pesan" 1 0
k=0
RS_TEST_VERIFY_KELOMPOK="lisensi-rentan,tidak-ada" bash "$ASLI" --daftar >/dev/null 2>&1 || k=$?
hasil "salah-satu-kelompok-tidak-dikenal-gagal" "$k" 2
k=0
bash "$ASLI" --bukan-opsi >/dev/null 2>&1 || k=$?
hasil "argumen-asing-ditolak" "$k" 2
semua="$(bash "$ASLI" --daftar | wc -l | tr -d ' ')"
satu="$(RS_TEST_VERIFY_KELOMPOK=lisensi-rentan bash "$ASLI" --daftar | wc -l | tr -d ' ')"
tak_kosong=1
if [ "$semua" -gt "$satu" ] && [ "$satu" -gt 0 ]; then tak_kosong=0; fi
hasil "pilihan-satu-kelompok-subset-dan-tidak-kosong" "$tak_kosong" 0
sama="$(RS_TEST_VERIFY_KELOMPOK=semua bash "$ASLI" --daftar | cksum)"
hasil "tanpa-pilihan-sama-dengan-semua" "$([ "$sama" = "$(bash "$ASLI" --daftar | cksum)" ] && echo 0 || echo 1)" 0

echo ""
echo "Lulus: $LULUS, gagal: $GAGAL"
if [ "$GAGAL" -ne 0 ] || [ "$LULUS" -eq 0 ]; then
  echo "HASIL: MERAH"
  exit 1
fi
echo "HASIL: HIJAU"
