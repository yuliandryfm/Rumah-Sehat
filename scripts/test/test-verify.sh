#!/usr/bin/env bash
# scripts/test/test-verify.sh: test perilaku scripts/verify.sh (F1-T06; catatan review F1-T05 butir a).
#
# Setiap fixture adalah salinan pohon kerja repo (berkas dilacak dan belum dilacak yang tidak diabaikan,
# tanpa .claude/; docs/ ikut karena test membaca registry dan data dictionary) di direktori sementara,
# dijadikan repo git sendiri, lalu diberi satu cacat. Direktori target cargo dipakai bersama semua fixture
# (CARGO_TARGET_DIR) supaya dependensi registry hanya dikompilasi sekali; crate workspace selalu dikompilasi
# ulang (lihat fixture_baru) sehingga tidak ada artefak basi antar-fixture.
# verify.sh salinan itu dijalankan dan hasilnya harus: kode keluar sesuai, "HASIL: MERAH", dan langkah yang
# ditargetkan tercatat GAGAL di ringkasan. Satu fixture bersih harus HIJAU dengan keempat pemindai LULUS.
#
# - Commit fixture memakai plumbing git (write-tree, commit-tree), jadi tidak menjalankan hook apa pun.
# - Butuh alat yang sama dengan verify.sh dan akses jaringan (advisory-db, crates.io, api.osv.dev).
# - Rahasia uji dibangun saat jalan dari potongan, supaya berkas ini sendiri tidak memicu gitleaks.
# - Portabel Linux dan macOS (bash 3.2). Tidak menerima argumen.

set -euo pipefail

if [ "$#" -ne 0 ]; then
  echo "test-verify.sh: tidak menerima argumen." >&2
  exit 2
fi

# Variabel GIT_* warisan (misalnya bila dijalankan dari git hook) akan mengarahkan perintah git fixture
# ke repo atau index lain; dibuang sebelum apa pun dijalankan.
for v in $(env | sed -n 's/^\(GIT_[A-Za-z0-9_]*\)=.*/\1/p'); do
  unset "$v"
done

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KERJA="$(mktemp -d "${TMPDIR:-/tmp}/test-verify.XXXXXX")"
trap 'rm -rf "$KERJA"' EXIT

export GIT_AUTHOR_NAME="fixture" GIT_AUTHOR_EMAIL="fixture@example.invalid"
export GIT_COMMITTER_NAME="fixture" GIT_COMMITTER_EMAIL="fixture@example.invalid"
export CARGO_TARGET_DIR="$KERJA/target"
# Tanpa kompilasi inkremental: setiap fixture dikompilasi ulang penuh, jadi cache inkremental hanya
# memakan ruang (kuota /tmp habis saat diuji). Tidak mengubah hasil lint atau test.
export CARGO_INCREMENTAL=0
# Cache sidik isi verify.sh (KEPUTUSAN-301) diarahkan ke direktori kerja test, tidak pernah ke cache
# developer. Di CI (CI=true) cache mati untuk semua fixture; bagian "Cache sidik isi" di bawah menyalakannya
# secara eksplisit dengan direktori cache sendiri.
export XDG_CACHE_HOME="$KERJA/cache"

LULUS=0
GAGAL=0
RINGKASAN=""

# Token berbentuk GitHub PAT (aturan gitleaks github-pat), bukan token sungguhan.
BAGIAN_A="gh""p_"
BAGIAN_B="Zq8XvT3mW1pL7rN4""kB9sD2fH6jC0yG5aE8uQ"
TOKEN_UJI="${BAGIAN_A}${BAGIAN_B}"

commit_semua() { # commit_semua <repo> <pesan>: stage semua lalu commit lewat plumbing (tanpa hook)
  local repo="$1" pesan="$2" pohon induk komit
  git -C "$repo" add -A
  pohon="$(git -C "$repo" write-tree)"
  if induk="$(git -C "$repo" rev-parse -q --verify HEAD)"; then
    komit="$(git -C "$repo" commit-tree --no-gpg-sign "$pohon" -p "$induk" -m "$pesan")"
  else
    komit="$(git -C "$repo" commit-tree --no-gpg-sign "$pohon" -m "$pesan")"
  fi
  git -C "$repo" update-ref HEAD "$komit"
}

tambah_dep_rentan() { # tambah_dep_rentan <repo>: smallvec 0.6.13 (RUSTSEC-2021-0003, SRC-363) di bawah [dependencies]
  # Seri 0.6 dipilih supaya tidak bentrok dengan smallvec 1.x yang sudah ada di graf (satu versi per seri semver).
  local berkas="$1/crates/foundation/Cargo.toml"
  awk '{ print } /^\[dependencies\]$/ { print "smallvec = \"=0.6.13\"" }' "$berkas" >"$berkas.baru"
  mv "$berkas.baru" "$berkas"
  grep -q '^smallvec = "=0.6.13"$' "$berkas"
  # Resolusi minimal: hanya entri baru ditambahkan ke Cargo.lock, versi lain tetap.
  (cd "$1" && cargo metadata --format-version 1 >/dev/null)
}

siapkan_murni() { # salinan pohon kerja repo ke $KERJA/murni, satu commit; verify.sh tidak pernah dijalankan di sini
  local tujuan="$KERJA/murni" berkas
  mkdir -p "$tujuan"
  (cd "$ROOT" && git ls-files -z --cached --others --exclude-standard) | while IFS= read -r -d '' berkas; do
    case "$berkas" in
      .claude/*) continue ;;
    esac
    if [ -f "$ROOT/$berkas" ]; then
      mkdir -p "$tujuan/$(dirname "$berkas")"
      cp -p "$ROOT/$berkas" "$tujuan/$berkas"
    fi
  done
  git -C "$tujuan" init -q
  commit_semua "$tujuan" "fixture: salinan repo"
  # Langkah migrasi-db mencari basis lewat `git merge-base HEAD main`; commit salinan menjadi basis itu.
  git -C "$tujuan" update-ref refs/heads/main HEAD
}

fixture_baru() { # fixture_baru <nama>: salin repo murni ke $KERJA/fixture (fixture sebelumnya dibuang), cetak path-nya
  # Satu path untuk semua fixture: artefak crate workspace di CARGO_TARGET_DIR bersama ditimpa, tidak menumpuk
  # (menumpuk per path menghabiskan kuota /tmp). cp tanpa -p memberi mtime baru ke semua berkas, sehingga
  # cargo selalu mengompilasi ulang crate workspace dan tidak memakai artefak basi dari fixture sebelumnya.
  # Nama fixture hanya dipakai untuk berkas log.
  rm -rf "$KERJA/fixture"
  cp -R "$KERJA/murni" "$KERJA/fixture"
  printf '%s' "$KERJA/fixture"
}

verify_di() { # verify_di <repo> <log> [VAR=nilai ...] [-- argumen verify ...]: cetak kode keluar verify.sh
  local repo="$1" log="$2" kode=0
  local envs=() args=()
  shift 2
  while [ "$#" -gt 0 ] && [ "$1" != "--" ]; do
    envs+=("$1")
    shift
  done
  if [ "$#" -gt 0 ]; then
    shift
  fi
  args=(${@+"$@"})
  (cd "$repo" && env ${envs[@]+"${envs[@]}"} bash scripts/verify.sh ${args[@]+"${args[@]}"}) >"$log" 2>&1 || kode=$?
  printf '%s' "$kode"
}

nilai() { # nilai <nama> <kode-diharapkan> <kode-aktual> <log> [pola-ERE-wajib ...]
  local nama="$1" harap="$2" aktual="$3" log="$4" pola ok=1
  shift 4
  if [ "$aktual" != "$harap" ]; then
    echo "  kode keluar $aktual, diharapkan $harap" >&2
    ok=0
  fi
  for pola in "$@"; do
    if ! grep -Eq -- "$pola" "$log"; then
      echo "  pola tidak ditemukan di keluaran: $pola" >&2
      ok=0
    fi
  done
  if [ "$ok" -eq 1 ]; then
    LULUS=$((LULUS + 1))
    RINGKASAN="${RINGKASAN}  LULUS  ${nama}"$'\n'
    echo "LULUS  $nama"
  else
    GAGAL=$((GAGAL + 1))
    RINGKASAN="${RINGKASAN}  GAGAL  ${nama}"$'\n'
    echo "GAGAL  $nama; ringkasan keluaran verify.sh:" >&2
    sed -n '/== Ringkasan verify.sh ==/,$p' "$log" >&2
    echo "  baris galat (maks. 40):" >&2
    grep -E 'panicked|FAILED|^error|GAGAL|tidak ditemukan|ditolak' "$log" | sed -n '1,40p' >&2 || true
  fi
}

merah() { # merah <nama> <kode-aktual> <log> <langkah-gagal ...>: kode 1, MERAH, dan setiap langkah GAGAL
  local nama="$1" kode="$2" log="$3" langkah pola
  shift 3
  pola=("HASIL: MERAH")
  for langkah in "$@"; do
    pola+=("^  GAGAL +${langkah}")
  done
  nilai "$nama" 1 "$kode" "$log" "${pola[@]}"
}

path_tanpa() { # path_tanpa <alat ...>: cetak PATH baru tanpa alat-alat itu (direktorinya diganti tiruan)
  local d f nama_f nama cocok i=0 baru="" tiruan sisa="$PATH:"
  while [ -n "$sisa" ]; do
    d="${sisa%%:*}"
    sisa="${sisa#*:}"
    cocok=0
    for nama in "$@"; do
      if [ -n "$d" ] && [ -e "$d/$nama" ]; then
        cocok=1
      fi
    done
    if [ "$cocok" -eq 1 ]; then
      i=$((i + 1))
      tiruan="$KERJA/path-tanpa-$i"
      mkdir -p "$tiruan"
      for f in "$d"/*; do
        nama_f="$(basename "$f")"
        cocok=0
        for nama in "$@"; do
          if [ "$nama_f" = "$nama" ]; then
            cocok=1
          fi
        done
        if [ "$cocok" -eq 0 ]; then
          ln -s "$f" "$tiruan/$nama_f"
        fi
      done
      baru="${baru:+$baru:}$tiruan"
    elif [ -n "$d" ]; then
      baru="${baru:+$baru:}$d"
    fi
  done
  printf '%s' "$baru"
}

echo "test-verify.sh: menguji $ROOT/scripts/verify.sh (direktori kerja $KERJA)"

# --- Test unit penyaringan kelas sidik (cepat, tanpa Docker; review T1 KEPUTUSAN-301, B-1) ---
k=0
bash "$ROOT/scripts/test/test-verify-sidik.sh" >"$KERJA/sidik-unit.log" 2>&1 || k=$?
nilai "unit-saring-kelas-sidik" 0 "$k" "$KERJA/sidik-unit.log" "^HASIL: HIJAU$"

siapkan_murni

# --- Fixture bersih: harus HIJAU, keempat pemindai berjalan dan lulus ---
r="$(fixture_baru bersih)"
k="$(verify_di "$r" "$KERJA/bersih.log")"
nilai "bersih-hijau" 0 "$k" "$KERJA/bersih.log" "HASIL: HIJAU" "^  LULUS +cargo-deny" "^  LULUS +cargo-audit" "^  LULUS +gitleaks" "^  LULUS +osv-scanner"

# --- Lingkungan git hook: GIT_INDEX_FILE diset (seperti pre-commit); index repo tidak boleh berubah ---
# Regresi F1-T06: cargo-audit yang mewarisi GIT_INDEX_FILE menimpa index repo dengan index advisory-db.
r="$(fixture_baru env-hook)"
printf '\n' >>"$r/.env.example"
git -C "$r" add .env.example
# Yang dibandingkan isi index (path, mode, hash blob), bukan byte berkasnya: git boleh menulis ulang
# info stat di index saat diff/ls-files.
sebelum="$(git -C "$r" ls-files -s | cksum)"
k="$(verify_di "$r" "$KERJA/env-hook.log" "GIT_INDEX_FILE=$r/.git/index")"
if [ "$(git -C "$r" ls-files -s | cksum)" = "$sebelum" ]; then
  echo "INDEX-FIXTURE: TIDAK BERUBAH" >>"$KERJA/env-hook.log"
else
  echo "INDEX-FIXTURE: BERUBAH" >>"$KERJA/env-hook.log"
fi
nilai "hook-GIT_INDEX_FILE-index-utuh" 0 "$k" "$KERJA/env-hook.log" "HASIL: HIJAU" "^INDEX-FIXTURE: TIDAK BERUBAH$"

# --- Argumen ditolak ---
r="$(fixture_baru argumen)"
k="$(verify_di "$r" "$KERJA/argumen.log" -- --quick)"
nilai "argumen-ditolak" 2 "$k" "$KERJA/argumen.log" "tidak menerima argumen"

# --- Lingkungan ---
r="$(fixture_baru env-rustflags)"
k="$(verify_di "$r" "$KERJA/env-rustflags.log" RUSTFLAGS=-Awarnings)"
merah "env-RUSTFLAGS" "$k" "$KERJA/env-rustflags.log" "lingkungan"

r="$(fixture_baru env-gitleaks)"
k="$(verify_di "$r" "$KERJA/env-gitleaks.log" GITLEAKS_CONFIG_TOML='title = "kosong"')"
merah "env-GITLEAKS_CONFIG_TOML" "$k" "$KERJA/env-gitleaks.log" "lingkungan"

r="$(fixture_baru cargo-config)"
mkdir -p "$r/.cargo"
printf '[build]\nrustflags = ["-Awarnings"]\n' >"$r/.cargo/config.toml"
commit_semua "$r" "fixture: .cargo/config.toml"
k="$(verify_di "$r" "$KERJA/cargo-config.log")"
merah "cargo-config-di-repo" "$k" "$KERJA/cargo-config.log" "lingkungan"

# --- Rust ---
r="$(fixture_baru fmt)"
printf '\npub fn  salah_format( ) {}\n' >>"$r/crates/foundation/src/lib.rs"
commit_semua "$r" "fixture: format"
k="$(verify_di "$r" "$KERJA/fmt.log")"
merah "cargo-fmt" "$k" "$KERJA/fmt.log" "cargo-fmt"

r="$(fixture_baru clippy)"
printf '\npub fn peringatan() {\n    let tidak_dipakai = 1;\n}\n' >>"$r/crates/foundation/src/lib.rs"
commit_semua "$r" "fixture: peringatan"
k="$(verify_di "$r" "$KERJA/clippy.log")"
merah "cargo-clippy" "$k" "$KERJA/clippy.log" "cargo-clippy"

r="$(fixture_baru test)"
printf '\npub fn satu() -> i32 {\n    1\n}\n\n#[cfg(test)]\nmod uji_fixture {\n    #[test]\n    fn harus_gagal() {\n        assert_eq!(super::satu(), 2);\n    }\n}\n' >>"$r/crates/foundation/src/lib.rs"
commit_semua "$r" "fixture: test gagal"
k="$(verify_di "$r" "$KERJA/test.log")"
merah "cargo-test" "$k" "$KERJA/test.log" "cargo-test"

# --- gitleaks ---
r="$(fixture_baru gitleaks-riwayat)"
printf 'kredensial = "%s"\n' "$TOKEN_UJI" >"$r/bocor.txt"
commit_semua "$r" "fixture: rahasia ter-commit"
rm "$r/bocor.txt"
commit_semua "$r" "fixture: rahasia dihapus dari pohon kerja, tetap ada di riwayat"
k="$(verify_di "$r" "$KERJA/gitleaks-riwayat.log")"
merah "gitleaks-riwayat" "$k" "$KERJA/gitleaks-riwayat.log" "gitleaks"

r="$(fixture_baru gitleaks-staged)"
printf 'kredensial = "%s"\n' "$TOKEN_UJI" >"$r/bocor.txt"
git -C "$r" add bocor.txt
k="$(verify_di "$r" "$KERJA/gitleaks-staged.log")"
merah "gitleaks-staged" "$k" "$KERJA/gitleaks-staged.log" "gitleaks"

r="$(fixture_baru gitleaks-unstaged)"
printf 'KREDENSIAL=%s\n' "$TOKEN_UJI" >>"$r/.env.example"
k="$(verify_di "$r" "$KERJA/gitleaks-unstaged.log")"
merah "gitleaks-belum-staged" "$k" "$KERJA/gitleaks-unstaged.log" "gitleaks"

r="$(fixture_baru gitleaks-allow)"
printf 'kredensial = "%s" # gitleaks:allow\n' "$TOKEN_UJI" >"$r/bocor.txt"
commit_semua "$r" "fixture: rahasia dengan komentar gitleaks:allow"
k="$(verify_di "$r" "$KERJA/gitleaks-allow.log")"
merah "gitleaks-komentar-allow-tidak-berlaku" "$k" "$KERJA/gitleaks-allow.log" "gitleaks"

# --- Konfigurasi pemindai di luar berkas resmi (review keamanan F1-T06, B-1) ---
# .gitleaks.toml liar yang membungkam semua aturan, belum dilacak, di subdirektori; ada rahasia ter-stage.
r="$(fixture_baru gitleaks-config-liar)"
printf '[[rules]]\nid = "x"\nregex = %s\n' "'''ZZZTIDAKPERNAHCOCOKZZZ'''" >"$r/crates/.gitleaks.toml"
printf 'kredensial = "%s"\n' "$TOKEN_UJI" >"$r/bocor.txt"
git -C "$r" add bocor.txt
k="$(verify_di "$r" "$KERJA/gitleaks-config-liar.log")"
merah "gitleaks-config-liar" "$k" "$KERJA/gitleaks-config-liar.log" "gitleaks"
nilai "gitleaks-config-liar-pesan" 1 "$k" "$KERJA/gitleaks-config-liar.log" "konfigurasi gitleaks di luar berkas resmi ditolak"

# Berkas .gitleaks.toml resmi tidak ada.
r="$(fixture_baru gitleaks-config-hilang)"
rm "$r/.gitleaks.toml"
commit_semua "$r" "fixture: .gitleaks.toml resmi dihapus"
k="$(verify_di "$r" "$KERJA/gitleaks-config-hilang.log")"
merah "gitleaks-config-resmi-hilang" "$k" "$KERJA/gitleaks-config-hilang.log" "gitleaks"

# osv-scanner.toml liar di samping lockfile, dilacak, mengabaikan advisory dependensi rentan.
r="$(fixture_baru osv-config-liar)"
tambah_dep_rentan "$r"
printf '[[IgnoredVulns]]\nid = "RUSTSEC-2021-0003"\nreason = "x"\n[[IgnoredVulns]]\nid = "GHSA-43w2-9j62-hq99"\nreason = "x"\n[[IgnoredVulns]]\nid = "CVE-2021-25900"\nreason = "x"\n' >"$r/crates/foundation/osv-scanner.toml"
commit_semua "$r" "fixture: osv-scanner.toml liar"
k="$(verify_di "$r" "$KERJA/osv-config-liar.log")"
merah "osv-config-liar" "$k" "$KERJA/osv-config-liar.log" "osv-scanner"
nilai "osv-config-liar-pesan" 1 "$k" "$KERJA/osv-config-liar.log" "konfigurasi osv-scanner di luar berkas resmi ditolak"

# .cargo/audit.toml belum dilacak yang mengabaikan advisory.
r="$(fixture_baru audit-toml)"
mkdir -p "$r/.cargo"
printf '[advisories]\nignore = ["RUSTSEC-2021-0003"]\n' >"$r/.cargo/audit.toml"
k="$(verify_di "$r" "$KERJA/audit-toml.log")"
merah "cargo-audit-toml-di-repo" "$k" "$KERJA/audit-toml.log" "lingkungan"

# Lockfile yang tidak dikenali verify.sh (review F1-T06, non-blokir 3).
r="$(fixture_baru lockfile-asing)"
mkdir -p "$r/web"
printf '{"lockfileVersion": 1}\n' >"$r/web/bun.lock"
commit_semua "$r" "fixture: lockfile tidak dikenali"
k="$(verify_di "$r" "$KERJA/lockfile-asing.log")"
merah "lockfile-tidak-dikenali" "$k" "$KERJA/lockfile-asing.log" "osv-scanner"
nilai "lockfile-tidak-dikenali-pesan" 1 "$k" "$KERJA/lockfile-asing.log" "lockfile yang tidak dikenali verify.sh"

# --- cargo-deny ---
r="$(fixture_baru deny-hilang)"
rm "$r/deny.toml"
commit_semua "$r" "fixture: deny.toml dihapus"
k="$(verify_di "$r" "$KERJA/deny-hilang.log")"
merah "deny.toml-hilang" "$k" "$KERJA/deny-hilang.log" "cargo-deny"

r="$(fixture_baru lisensi)"
mkdir -p "$r/crates/zz-lisensi-uji/src"
printf '[package]\nname = "zz-lisensi-uji"\nversion = "0.1.0"\nedition = "2024"\nlicense = "GPL-3.0-only"\n\n[dependencies]\n' >"$r/crates/zz-lisensi-uji/Cargo.toml"
printf '//! Fixture lisensi di luar daftar izin.\n' >"$r/crates/zz-lisensi-uji/src/lib.rs"
(cd "$r" && cargo metadata --offline --format-version 1 >/dev/null)
commit_semua "$r" "fixture: crate workspace berlisensi GPL tanpa publish = false"
k="$(verify_di "$r" "$KERJA/lisensi.log")"
merah "lisensi-di-luar-daftar-izin" "$k" "$KERJA/lisensi.log" "cargo-deny"

# --- Dependensi rentan: smallvec 0.6.13, RUSTSEC-2021-0003 (SRC-363); butuh jaringan ---
r="$(fixture_baru rentan)"
tambah_dep_rentan "$r"
commit_semua "$r" "fixture: dependensi dengan advisory"
k="$(verify_di "$r" "$KERJA/rentan.log")"
merah "dependensi-rentan" "$k" "$KERJA/rentan.log" "cargo-deny" "cargo-audit" "osv-scanner"

# --- Alat tidak ada di PATH atau versinya lain: GAGAL, bukan dilewati ---
r="$(fixture_baru alat-hilang)"
k="$(verify_di "$r" "$KERJA/alat-hilang.log" "PATH=$(path_tanpa gitleaks osv-scanner cargo-deny cargo-audit)")"
merah "alat-tidak-ada-di-PATH" "$k" "$KERJA/alat-hilang.log" "cargo-deny" "cargo-audit" "gitleaks" "osv-scanner"
nilai "alat-tidak-ada-di-PATH-pesan" 1 "$k" "$KERJA/alat-hilang.log" "gitleaks tidak ditemukan di PATH"

r="$(fixture_baru alat-versi)"
mkdir -p "$KERJA/shim"
printf '#!/bin/sh\necho 8.0.0\n' >"$KERJA/shim/gitleaks"
chmod +x "$KERJA/shim/gitleaks"
k="$(verify_di "$r" "$KERJA/alat-versi.log" "PATH=$KERJA/shim:$PATH")"
merah "alat-versi-lain" "$k" "$KERJA/alat-versi.log" "gitleaks"
nilai "alat-versi-lain-pesan" 1 "$k" "$KERJA/alat-versi.log" "gitleaks aktif '8.0.0', dibutuhkan"

# --- Web (F1-T39): workspace tidak lengkap, atau package.json tidak sinkron dengan lockfile ---
r="$(fixture_baru artefak)"
rm -rf "$r/web"
mkdir -p "$r/web"
printf '{"name": "fixture"}\n' >"$r/web/package.json"
commit_semua "$r" "fixture: web/package.json tanpa pnpm-workspace.yaml"
k="$(verify_di "$r" "$KERJA/artefak.log")"
merah "web-tanpa-workspace" "$k" "$KERJA/artefak.log" "web"
nilai "web-tanpa-workspace-pesan" 1 "$k" "$KERJA/artefak.log" "harus ada bersamaan"

r="$(fixture_baru web-lock-basi)"
sed 's/"vitest": "5.1.1"/"vitest": "5.1.0"/' "$r/web/package.json" >"$r/web/package.json.baru"
mv "$r/web/package.json.baru" "$r/web/package.json"
commit_semua "$r" "fixture: package.json web tidak sinkron dengan pnpm-lock.yaml"
k="$(verify_di "$r" "$KERJA/web-lock-basi.log")"
merah "web-lockfile-tidak-sinkron" "$k" "$KERJA/web-lock-basi.log" "web-install"

# Token warna di bawah syarat kontras (F1-T40): skrip kontras harus MERAH.
r="$(fixture_baru web-kontras)"
sed 's/--c-text-secondary: #5c6b6f;/--c-text-secondary: #aaaaaa;/' "$r/web/packages/ui/src/tokens.css" >"$r/web/packages/ui/src/tokens.css.baru"
mv "$r/web/packages/ui/src/tokens.css.baru" "$r/web/packages/ui/src/tokens.css"
commit_semua "$r" "fixture: text.secondary di bawah kontras"
k="$(verify_di "$r" "$KERJA/web-kontras.log")"
merah "web-kontras-di-bawah-syarat" "$k" "$KERJA/web-kontras.log" "web-kontras"

# --- Berkas migrasi baru tanpa penjaga (langkah migrasi-db, F1-T15) ---
r="$(fixture_baru migrasi)"
mkdir -p "$r/db/migrations/app"
printf -- '-- fixture\n' >"$r/db/migrations/app/99990101000000_uji.sql"
commit_semua "$r" "fixture: migrasi tanpa penjaga"
k="$(verify_di "$r" "$KERJA/migrasi.log")"
merah "migrasi-tanpa-penjaga" "$k" "$KERJA/migrasi.log" "migrasi-db"

# --- Cache sidik isi (KEPUTUSAN-301) ---
# Cache dinyalakan eksplisit (CI dan GITHUB_ACTIONS dikosongkan) dengan direktori cache baru, supaya hasilnya
# sama di lokal dan CI. Langkah yang dapat di-cache: 16 (lihat jalankan_sidik di verify.sh).
CACHE_UJI="$KERJA/cache-uji"
rm -rf "$CACHE_UJI"
NYALA=("CI=" "GITHUB_ACTIONS=" "RS_VERIFY_TANPA_CACHE=" "XDG_CACHE_HOME=$CACHE_UJI")
JUMLAH_CACHE=16

tandai_cache() { # tandai_cache <log> <label> <ERE>: tambahkan "<label>: ADA/TIDAK ADA" ke log
  if grep -Eq -- "$3" "$1"; then
    echo "$2: ADA" >>"$1"
  else
    echo "$2: TIDAK ADA" >>"$1"
  fi
}

# Run pertama: semua langkah berjalan, cache masih kosong.
r="$(fixture_baru cache-awal)"
k="$(verify_di "$r" "$KERJA/cache-awal.log" "${NYALA[@]}")"
nilai "cache-run-pertama-jalan-penuh" 0 "$k" "$KERJA/cache-awal.log" "HASIL: HIJAU" "^cache sidik: aktif" \
  "dari cache sidik: 0\)" "^  LULUS +cargo-test$" "^  LULUS +web-build$"

# Run kedua: pada salinan segar, run pertama membuat berkas yang diabaikan tetapi ikut sidik (next-env.d.ts dari
# next build, dibaca typecheck), jadi langkah yang sidiknya dihitung sebelum berkas itu ada berjalan lagi.
k="$(verify_di "$r" "$KERJA/cache-kedua.log" "${NYALA[@]}")"
nilai "cache-run-kedua-hijau" 0 "$k" "$KERJA/cache-kedua.log" "HASIL: HIJAU" "^  LULUS +cargo-fmt \(cache, sidik [0-9a-f]{8}\)$"

# Run ketiga tanpa perubahan: semua langkah yang dapat di-cache diambil dari cache; pemindai jaringan,
# gitleaks, dan pemeriksaan lingkungan tetap jalan.
k="$(verify_di "$r" "$KERJA/cache-ketiga.log" "${NYALA[@]}")"
nilai "cache-run-ketiga-dari-cache" 0 "$k" "$KERJA/cache-ketiga.log" "HASIL: HIJAU" \
  "dari cache sidik: ${JUMLAH_CACHE}\)" "^==> \[cargo-test\] LULUS \(cache, sidik [0-9a-f]{8}\)$" \
  "^  LULUS +schemathesis \(cache, sidik [0-9a-f]{8}\)$" "^  LULUS +web-build \(cache, sidik [0-9a-f]{8}\)$" \
  "^  LULUS +cargo-deny$" "^  LULUS +cargo-audit$" "^  LULUS +gitleaks$" "^  LULUS +osv-scanner$" "^  LULUS +web-install$"

# Catatan cache rusak atau dibuat tangan dianggap tidak ada: cargo-fmt dikosongkan (terpotong), web-lint
# diganti berkas tulisan tangan berformat benar tetapi MAC salah, web-test diganti berkas kosong hasil touch
# (nama = sidik). Ketiganya dijalankan lagi; langkah lain tetap dari cache.
for f in "$CACHE_UJI"/v1/cargo-fmt/*; do : >"$f"; done
for f in "$CACHE_UJI"/v1/web-lint/*; do
  printf 'ruang-sehat-verify-cache v1\nlangkah web-lint\nsidik %s\nwaktu %s\nmac %064d\n' "$(basename "$f")" "$(date +%s)" 0 >"$f"
done
for f in "$CACHE_UJI"/v1/web-test/*; do rm -f "$f" && touch "$f"; done
k="$(verify_di "$r" "$KERJA/cache-rusak.log" "${NYALA[@]}")"
nilai "cache-rusak-atau-palsu-diabaikan" 0 "$k" "$KERJA/cache-rusak.log" "HASIL: HIJAU" \
  "dari cache sidik: $((JUMLAH_CACHE - 3))\)" "^==> \[cargo-fmt\] cargo fmt --all --check$" \
  "^==> \[web-lint\] di_web pnpm run lint$" "^==> \[web-test\] di_web pnpm run test$" \
  "^  LULUS +cargo-test \(cache, sidik [0-9a-f]{8}\)$"

# CI=true: cache mati walau catatan lulus untuk isi yang sama ada.
k="$(verify_di "$r" "$KERJA/cache-ci.log" "CI=true" "GITHUB_ACTIONS=" "RS_VERIFY_TANPA_CACHE=" "XDG_CACHE_HOME=$CACHE_UJI")"
tandai_cache "$KERJA/cache-ci.log" "CACHE-DIPAKAI" "\(cache, sidik"
nilai "cache-CI-true-tanpa-cache" 0 "$k" "$KERJA/cache-ci.log" "HASIL: HIJAU" "^cache sidik: mati \(CI" \
  "dari cache sidik: 0\)" "^  LULUS +cargo-test$" "^CACHE-DIPAKAI: TIDAK ADA$"

# scripts/verify-penuh.sh: cache mati, semua langkah berjalan.
k=0
(cd "$r" && env "${NYALA[@]}" bash scripts/verify-penuh.sh) >"$KERJA/cache-penuh.log" 2>&1 || k=$?
tandai_cache "$KERJA/cache-penuh.log" "CACHE-DIPAKAI" "\(cache, sidik"
nilai "verify-penuh-tanpa-cache" 0 "$k" "$KERJA/cache-penuh.log" "HASIL: HIJAU" \
  "^cache sidik: mati \(RS_VERIFY_TANPA_CACHE" "dari cache sidik: 0\)" "^CACHE-DIPAKAI: TIDAK ADA$"
k=0
(cd "$r" && bash scripts/verify-penuh.sh --tanpa-langkah) >"$KERJA/cache-penuh-arg.log" 2>&1 || k=$?
nilai "verify-penuh-argumen-ditolak" 2 "$k" "$KERJA/cache-penuh-arg.log" "tidak menerima argumen"

# Perubahan 1 byte (spasi menjadi tab, belum di-commit) di masukan langkah yang sudah di-cache lulus:
# cache batal, cargo-fmt dijalankan dan MERAH. Langkah kelas web tidak terpengaruh (tetap dari cache).
r="$(fixture_baru cache-1-byte)"
tab="$(printf '\t')"
cp "$r/crates/foundation/src/lib.rs" "$KERJA/lib.rs.asli"
sed "s/^pub use sensitive::Sensitive;\$/pub${tab}use sensitive::Sensitive;/" "$KERJA/lib.rs.asli" >"$r/crates/foundation/src/lib.rs"
echo "BYTE-BERBEDA: $(cmp -l "$KERJA/lib.rs.asli" "$r/crates/foundation/src/lib.rs" | wc -l | tr -d ' ')" >"$KERJA/cache-1-byte.log.pra"
k="$(verify_di "$r" "$KERJA/cache-1-byte.log" "${NYALA[@]}")"
cat "$KERJA/cache-1-byte.log.pra" >>"$KERJA/cache-1-byte.log"
merah "cache-batal-perubahan-1-byte" "$k" "$KERJA/cache-1-byte.log" "cargo-fmt"
nilai "cache-batal-perubahan-1-byte-cakupan" 1 "$k" "$KERJA/cache-1-byte.log" "^BYTE-BERBEDA: 1$" \
  "^==> \[cargo-fmt\] cargo fmt --all --check$" "^  LULUS +web-lint \(cache, sidik [0-9a-f]{8}\)$"

# Fixture merah dengan cache lulus dari isi lain: tetap MERAH (langkah kelas Rust boleh dari cache).
r="$(fixture_baru cache-merah)"
sed 's/--c-text-secondary: #5c6b6f;/--c-text-secondary: #aaaaaa;/' "$r/web/packages/ui/src/tokens.css" >"$r/web/packages/ui/src/tokens.css.baru"
mv "$r/web/packages/ui/src/tokens.css.baru" "$r/web/packages/ui/src/tokens.css"
k="$(verify_di "$r" "$KERJA/cache-merah.log" "${NYALA[@]}")"
merah "cache-fixture-merah-tetap-merah" "$k" "$KERJA/cache-merah.log" "web-kontras"
nilai "cache-fixture-merah-rust-dari-cache" 1 "$k" "$KERJA/cache-merah.log" "^  LULUS +cargo-clippy \(cache, sidik [0-9a-f]{8}\)$"

# GAGAL tidak di-cache: isi merah yang sama dijalankan lagi, web-kontras berjalan lagi dan tetap GAGAL.
k="$(verify_di "$r" "$KERJA/cache-merah-2.log" "${NYALA[@]}")"
tandai_cache "$KERJA/cache-merah-2.log" "CACHE-KONTRAS" "web-kontras.*\(cache, sidik"
merah "cache-gagal-tidak-di-cache" "$k" "$KERJA/cache-merah-2.log" "web-kontras"
nilai "cache-gagal-tidak-di-cache-dijalankan" 1 "$k" "$KERJA/cache-merah-2.log" \
  "^==> \[web-kontras\] di_web pnpm run kontras$" "^CACHE-KONTRAS: TIDAK ADA$"

# Tanpa rujukan docs/ (review T1 B-1): di salinan ini setiap "docs" di berkas .rs, scripts/db-ephemeral.sh, dan
# web/ diganti "dokx" (panjang sama, format tidak berubah), sehingga rujukan docs kelas rust dan web kosong.
# Test yang membaca docs/ ikut gagal (verify MERAH, kode 1); yang dinilai hanya cargo-fmt dan web-lint.
# Run 1-3 mengisi cache (run 3: web-lint dari cache setelah next-env.d.ts dibuat web-build), lalu satu berkas
# .rs dan satu berkas web diubah: kedua langkah wajib dijalankan ulang, bukan dari cache.
r="$(fixture_baru cache-tanpa-rujukan)"
(cd "$r" && git ls-files -z -- '*.rs' scripts/db-ephemeral.sh web) | while IFS= read -r -d '' berkas; do
  if [ -f "$r/$berkas" ] && grep -Iq 'docs' "$r/$berkas"; then
    sed 's/docs/dokx/g' "$r/$berkas" >"$r/$berkas.baru"
    cat "$r/$berkas.baru" >"$r/$berkas"
    rm -f "$r/$berkas.baru"
  fi
done
if (cd "$r" && git ls-files -z -- '*.rs' scripts/db-ephemeral.sh web | xargs -0 grep -Il 'docs' >/dev/null 2>&1); then
  echo "RUJUKAN-DOCS: MASIH ADA" >"$KERJA/cache-tanpa-rujukan.pra"
else
  echo "RUJUKAN-DOCS: KOSONG" >"$KERJA/cache-tanpa-rujukan.pra"
fi
for i in 1 2; do
  verify_di "$r" "$KERJA/cache-tanpa-rujukan-$i.log" "${NYALA[@]}" >/dev/null
done
k="$(verify_di "$r" "$KERJA/cache-tanpa-rujukan-3.log" "${NYALA[@]}")"
cat "$KERJA/cache-tanpa-rujukan.pra" >>"$KERJA/cache-tanpa-rujukan-3.log"
# Kode keluar tidak dinilai (verify MERAH karena test yang membaca docs/ gagal); yang dinilai pola keluaran.
nilai "cache-tanpa-rujukan-docs-dapat-di-cache" "$k" "$k" "$KERJA/cache-tanpa-rujukan-3.log" "^RUJUKAN-DOCS: KOSONG$" \
  "^  LULUS +cargo-fmt \(cache, sidik [0-9a-f]{8}\)$" "^  LULUS +web-lint \(cache, sidik [0-9a-f]{8}\)$"
printf '// ubah sidik\n' >>"$r/crates/foundation/src/lib.rs"
printf '// ubah sidik\n' >>"$r/web/eslint.config.js"
k="$(verify_di "$r" "$KERJA/cache-tanpa-rujukan-4.log" "${NYALA[@]}")"
tandai_cache "$KERJA/cache-tanpa-rujukan-4.log" "CACHE-FMT" "cargo-fmt \(cache, sidik"
tandai_cache "$KERJA/cache-tanpa-rujukan-4.log" "CACHE-LINT" "web-lint \(cache, sidik"
nilai "cache-tanpa-rujukan-docs-ubah-berkas-jalan-ulang" "$k" "$k" "$KERJA/cache-tanpa-rujukan-4.log" \
  "^==> \[cargo-fmt\] cargo fmt --all --check$" "^==> \[web-lint\] di_web pnpm run lint$" \
  "^CACHE-FMT: TIDAK ADA$" "^CACHE-LINT: TIDAK ADA$"

echo ""
echo "== Ringkasan test-verify.sh =="
printf '%s' "$RINGKASAN"
echo "Lulus: $LULUS, gagal: $GAGAL"
if [ "$GAGAL" -ne 0 ] || [ "$LULUS" -eq 0 ]; then
  echo "HASIL: MERAH"
  exit 1
fi
echo "HASIL: HIJAU"
