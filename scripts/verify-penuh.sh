#!/usr/bin/env bash
# scripts/verify-penuh.sh: verifikasi penuh tanpa cache sidik isi (KEPUTUSAN-301), untuk jadwal tiap malam.
#
# Menjalankan scripts/verify.sh dengan RS_VERIFY_TANPA_CACHE=1: setiap langkah dijalankan, tidak ada hasil
# yang dibaca dari atau ditulis ke cache. Variabel itu hanya mematikan cache (menambah langkah yang berjalan),
# tidak dapat dipakai untuk melewati langkah. Kode keluar sama dengan verify.sh. Tidak menerima argumen.
#
# Gunanya: menangkap hal yang tidak tercakup sidik isi, misalnya test yang bergantung pada tanggal, image
# Docker atau paket yang berubah di luar repo, dan konfigurasi di luar repo (~/.cargo).
#
# Penjadwalan (tidak dipasang otomatis). Contoh crontab pengguna (`crontab -e`), tiap malam pukul 02.30
# waktu mesin, dengan shell login supaya PATH memuat cargo, pnpm, dan pemindai; sesuaikan path checkout:
#   30 2 * * * bash -lc 'cd "$HOME/ruang-sehat/Rumah-Sehat" && bash scripts/verify-penuh.sh' >>"$HOME/.cache/ruang-sehat-verify/malam.log" 2>&1
# Alternatif: systemd user timer (`systemctl --user`) atau launchd di macOS dengan perintah yang sama.
# Hasil MERAH di log malam diperlakukan seperti verify merah: dilaporkan dan diperbaiki, bukan diabaikan.

set -euo pipefail

if [ "$#" -ne 0 ]; then
  echo "verify-penuh.sh: tidak menerima argumen (diberikan: $*)." >&2
  exit 2
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "verify-penuh.sh: $(date -u +%Y-%m-%dT%H:%M:%SZ), verifikasi penuh tanpa cache sidik"
export RS_VERIFY_TANPA_CACHE=1
exec bash "$ROOT/scripts/verify.sh"
