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

set -euo pipefail

if [ "$#" -ne 0 ]; then
  echo "verify.sh: tidak menerima argumen (diberikan: $*)." >&2
  exit 2
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

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

pakai_sqlx() { # benar bila workspace sudah memakai sqlx
  [ -d .sqlx ] && return 0
  [ -f Cargo.lock ] && grep -q '^name = "sqlx"$' Cargo.lock
}

echo "verify.sh: $ROOT"

# --- Langkah 0: lingkungan tidak boleh mengubah hasil (review keamanan F1-T05, B-1) ---
# Variabel berikut dapat mematikan lint atau mengganti runner test sehingga hasil hijau palsu.
# Ditolak (GAGAL), tidak di-unset diam-diam. Konfigurasi cargo di luar repo (~/.cargo/config.toml
# dan .cargo/config.toml di direktori induk repo) hanya tertutup oleh CI di lingkungan bersih (F1-T06).
VAR_TERLARANG="$(env | grep -E '^(RUSTFLAGS|CARGO_ENCODED_RUSTFLAGS|RUSTDOCFLAGS|RUSTC|RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER|CARGO_TARGET_[A-Za-z0-9_]*_RUNNER|CARGO_ALIAS_[A-Za-z0-9_]*|CARGO_BUILD_[A-Za-z0-9_]*|CARGO_PROFILE_[A-Za-z0-9_]*)=' | cut -d= -f1 | tr '\n' ' ' || true)"
if [ -n "$VAR_TERLARANG" ]; then
  gagal "lingkungan" "variabel yang dapat mengubah hasil verifikasi diset: ${VAR_TERLARANG}; hapus dari lingkungan lalu jalankan ulang"
fi
if [ -e .cargo/config ] || [ -e .cargo/config.toml ]; then
  gagal "lingkungan" ".cargo/config(.toml) ada di repo tetapi belum disetujui sebagai bagian verifikasi; perbarui scripts/verify.sh (perlu persetujuan user)"
fi

# --- Langkah 1: Rust (docs/05-stack.md §6 butir 1) ---
if [ ! -f Cargo.toml ]; then
  gagal "rust" "Cargo.toml (workspace, KEPUTUSAN-89) tidak ada"
elif ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
  gagal "rust" "cargo/rustc tidak ditemukan di PATH"
else
  jalankan "rust-toolchain" periksa_toolchain
  jalankan "cargo-fmt" cargo fmt --all --check
  jalankan "cargo-clippy" cargo clippy --workspace --all-targets --locked -- -D warnings
  jalankan "cargo-test" cargo test --workspace --locked
fi

if pakai_sqlx; then
  gagal "sqlx-prepare" "workspace memakai sqlx tetapi 'cargo sqlx prepare --check --workspace' belum diterapkan (task belum ditetapkan di phase plan §7.3); perbarui scripts/verify.sh (perlu persetujuan user)"
else
  belum "sqlx-prepare" "task pertama yang memakai sqlx; belum ditetapkan di phase plan §7.3"
fi
belum "cargo-deny" "F1-T06" "deny.toml"
belum "cargo-audit" "F1-T06"

# --- Langkah 2: Migrasi dan hak role (§6 butir 2; KEPUTUSAN-140, KEPUTUSAN-158) ---
belum "migrasi-db" "F1-T15" "db/migrations/*/*.sql" "db/roles/*.sql"

# --- Langkah 3: Kontrak API (§6 butir 3) ---
belum "openapi-sinkron" "F1-T11" "openapi/*.yaml" "openapi/*.yml" "openapi/*.json" "openapi/asyncapi/*.yaml" "openapi/asyncapi/*.yml" "openapi/asyncapi/*.json"
belum "registry-response-code" "F1-T09" "openapi/*.yaml" "openapi/*.yml" "openapi/*.json"
belum "schemathesis" "belum ditetapkan di phase plan §7.3" "openapi/*.yaml" "openapi/*.yml" "openapi/*.json"

# --- Langkah 4: Test arsitektur (§6 butir 4) ---
belum "test-arsitektur" "F1-T12" "apps/*/module-tables.toml"

# --- Langkah 5: Web (§6 butir 5) ---
belum "web" "F1-T39" "web/package.json" "web/pnpm-workspace.yaml" "web/*/package.json" "web/packages/*/package.json"

# --- Langkah 6: Mobile (§6 butir 6) ---
belum "mobile" "F1-T44" "mobile/pubspec.yaml" "mobile/apps/*/pubspec.yaml" "mobile/packages/*/pubspec.yaml"

# --- Langkah 7: Rahasia dan dependensi (§6 butir 7, §3.2) ---
belum "gitleaks" "F1-T06"
belum "osv-scanner" "F1-T06"

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
