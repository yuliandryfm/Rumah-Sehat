#!/usr/bin/env bash
# Stop hook: mencegah Claude menyatakan selesai saat verifikasi masih merah.
#
# Cara kerja:
# - Kalau hook ini sudah memicu kelanjutan sebelumnya (stop_hook_active), izinkan berhenti.
# - Kalau scripts/verify.sh belum ada (sebelum Tahap 8), izinkan berhenti.
# - Kalau tidak ada perubahan di luar folder docs/, izinkan berhenti.
# - Selain itu jalankan scripts/verify.sh. Kalau gagal, blokir (exit 2) dan kirim
#   potongan output ke Claude supaya diperbaiki.

INPUT="$(cat)"

if printf '%s' "$INPUT" | grep -Eq '"stop_hook_active"[[:space:]]*:[[:space:]]*true'; then
  exit 0
fi

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(pwd)}"
cd "$PROJECT_DIR" || exit 0

VERIFY="scripts/verify.sh"
if [ ! -f "$VERIFY" ]; then
  exit 0
fi

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  CHANGED="$(git status --porcelain | grep -v -E '^.. docs/' || true)"
  if [ -z "$CHANGED" ]; then
    exit 0
  fi
fi

OUTPUT="$(bash "$VERIFY" 2>&1)"
STATUS=$?

if [ $STATUS -ne 0 ]; then
  {
    echo "Verifikasi gagal (scripts/verify.sh keluar dengan kode $STATUS)."
    echo "Jangan menyatakan pekerjaan selesai. Perbaiki penyebabnya, lalu jalankan ulang scripts/verify.sh."
    echo "Potongan output terakhir:"
    printf '%s\n' "$OUTPUT" | tail -n 40
  } >&2
  exit 2
fi

exit 0
