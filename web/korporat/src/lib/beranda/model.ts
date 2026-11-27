import type { BadgeTone } from "@rs/ui";
import type { Muatan } from "../verifikasi/klien.ts";
import { DOKUMEN, LABEL_STATUS, hitungTerunggah, labelAlasan } from "../verifikasi/model.ts";

/**
 * Isi Beranda HR (CR-039, Q32): status perusahaan dan SATU tombol langkah berikutnya. Seluruhnya turunan dari
 * data yang sudah ada (`GET /v1/corporate/documents`: status, kelengkapan, dokumen); tidak ada angka, nama, atau
 * janji waktu yang dikarang. Ini bantuan tampilan: server tetap menentukan apa yang boleh dilakukan HR.
 */
export interface IsiBeranda {
  readonly kunci: "aktif" | "ditolak" | "belum-lengkap" | "diperiksa" | "netral";
  readonly judul: string;
  readonly status: { readonly label: string; readonly tone: BadgeTone };
  /** Kalimat pendek bahasa biasa: apa yang terjadi dan apa yang harus dilakukan. */
  readonly kalimat: readonly string[];
  /** Satu-satunya tombol utama. Tujuan selalu halaman yang sudah ada. */
  readonly tindakan: { readonly label: string; readonly href: "/verifikasi" | "/kelola-hr" };
}

export function isiBeranda(m: Muatan): IsiBeranda {
  if (m.jenis === "aktif") {
    return {
      kunci: "aktif",
      judul: "Perusahaan Anda sudah aktif",
      status: LABEL_STATUS.active,
      kalimat: [
        "Verifikasi selesai. Ajak rekan HR lain untuk ikut mengelola akun perusahaan.",
        "Menu Pesanan, Peserta, dan Tagihan segera hadir.",
      ],
      tindakan: { label: "Undang rekan HR", href: "/kelola-hr" },
    };
  }
  const r = m.ringkasan;
  // Dilebarkan ke string: server dapat menambah status baru yang belum dikenal klien (cabang akhir = netral).
  const status: string = r.status;
  if (r.status === "rejected") {
    return {
      kunci: "ditolak",
      judul: "Pengajuan perlu diperbaiki",
      status: LABEL_STATUS.rejected,
      kalimat: [`Alasan: ${labelAlasan(r.rejection_reason)}.`, "Perbaiki yang perlu, lalu ajukan ulang."],
      tindakan: { label: "Perbaiki dan ajukan ulang", href: "/verifikasi" },
    };
  }
  if (status === "pending_verification" && !r.complete) {
    return {
      kunci: "belum-lengkap",
      judul: "Lengkapi dokumen perusahaan",
      status: LABEL_STATUS.pending_verification,
      kalimat: [
        `${String(hitungTerunggah(r.documents))} dari ${String(DOKUMEN.length)} dokumen sudah diunggah.`,
        "Tim Ruang Sehat baru memeriksa pengajuan setelah ketiga dokumen terunggah.",
      ],
      tindakan: { label: "Unggah dokumen", href: "/verifikasi" },
    };
  }
  if (status === "pending_verification") {
    return {
      kunci: "diperiksa",
      judul: "Dokumen sedang diperiksa",
      status: LABEL_STATUS.pending_verification,
      kalimat: [
        "Tidak ada yang perlu Anda lakukan sekarang. Tim Ruang Sehat sedang memeriksa dokumen perusahaan Anda.",
        "Periksa halaman ini secara berkala untuk melihat hasilnya.",
      ],
      tindakan: { label: "Lihat status pemeriksaan", href: "/verifikasi" },
    };
  }
  // Status yang tidak dikenal (mis. nilai baru dari server): jangan menebak, arahkan ke rincian status.
  return {
    kunci: "netral",
    judul: "Status belum dapat ditampilkan",
    status: { label: "Status belum diketahui", tone: "neutral" },
    kalimat: ["Buka halaman status untuk melihat keadaan terbaru perusahaan Anda."],
    tindakan: { label: "Lihat status", href: "/verifikasi" },
  };
}
