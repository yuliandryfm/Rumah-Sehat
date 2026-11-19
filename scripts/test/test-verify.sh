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

# --- Artefak komponen ada tetapi langkahnya belum diterapkan ---
r="$(fixture_baru artefak)"
mkdir -p "$r/db/migrations/app"
printf -- '-- fixture\n' >"$r/db/migrations/app/0001_uji.sql"
commit_semua "$r" "fixture: migrasi tanpa langkah migrasi-db"
k="$(verify_di "$r" "$KERJA/artefak.log")"
merah "artefak-tanpa-langkah" "$k" "$KERJA/artefak.log" "migrasi-db"

echo ""
echo "== Ringkasan test-verify.sh =="
printf '%s' "$RINGKASAN"
echo "Lulus: $LULUS, gagal: $GAGAL"
if [ "$GAGAL" -ne 0 ] || [ "$LULUS" -eq 0 ]; then
  echo "HASIL: MERAH"
  exit 1
fi
echo "HASIL: HIJAU"
