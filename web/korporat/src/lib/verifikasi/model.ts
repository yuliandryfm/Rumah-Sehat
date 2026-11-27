import type { BadgeTone } from "@rs/ui";
import type { JenisDokumen, StatusVerifikasi } from "./klien.ts";

/**
 * Model layar status verifikasi dan dokumen perusahaan (F1-T103; US-1901 AC2, AC6; US-1902 AC3; wireframe K01
 * layar 7-8; brief web korporat KR-09, KR-10). Semua aturan di sini hanya bantuan tampilan: server tetap sumber
 * kebenaran (jenis berkas diperiksa dari isinya, ukuran, kelengkapan, dan status).
 */

/** Status yang dapat tampil: dua dari `GET /v1/corporate/documents`, `active` dari 409 `4090112`. */
export type StatusTampil = StatusVerifikasi | "active";

export const LABEL_STATUS: Record<StatusTampil, { readonly label: string; readonly tone: BadgeTone }> = {
  pending_verification: { label: "Menunggu verifikasi", tone: "warning" },
  rejected: { label: "Ditolak", tone: "danger" },
  active: { label: "Aktif", tone: "success" },
};

/** Urutan dan label tiga dokumen wajib (Q8 A; bukti kewenangan = surat kuasa/penugasan direksi, Q9 A). */
export const DOKUMEN: readonly { readonly jenis: JenisDokumen; readonly label: string; readonly bantuan: string }[] = [
  { jenis: "nib", label: "Dokumen NIB", bantuan: "Dokumen Nomor Induk Berusaha perusahaan." },
  { jenis: "npwp", label: "Dokumen NPWP", bantuan: "Kartu atau surat keterangan NPWP badan." },
  {
    jenis: "authority_proof",
    label: "Bukti kewenangan pendaftar",
    bantuan: "Surat kuasa atau surat penugasan dari direksi yang menunjuk Anda mendaftarkan perusahaan.",
  },
];

export const labelDokumen = (j: JenisDokumen): string => DOKUMEN.find((d) => d.jenis === j)?.label ?? j;

/**
 * Label kode alasan penolakan, sama dengan label daftar alasan di migrasi data
 * `db/migrations/app/20261124201000_corporate_rejection_reason_seed.sql` (daftar DRAF, F1-T104). Kode di luar
 * daftar tampil sebagai teks umum, tidak sebagai kode mentah.
 */
export const LABEL_ALASAN: Record<string, string> = {
  nib_document_invalid: "Dokumen NIB tidak terbaca, tidak sah, atau tidak sesuai NIB yang diisi",
  npwp_document_invalid: "Dokumen NPWP tidak terbaca, tidak sah, atau tidak sesuai NPWP yang diisi",
  company_data_mismatch: "Data perusahaan tidak sesuai dengan dokumen",
  authority_proof_insufficient: "Bukti kewenangan pendaftar tidak memadai",
};
export const ALASAN_UMUM = "Pengajuan belum memenuhi syarat verifikasi";

export function labelAlasan(kode: string | null | undefined): string {
  if (kode === null || kode === undefined || kode === "") return ALASAN_UMUM;
  return LABEL_ALASAN[kode] ?? ALASAN_UMUM;
}

/** Batas ukuran per jenis berkas (NFR-SEC-08 PRD v1.3; KEPUTUSAN-346): foto 5 MiB, PDF 10 MiB. */
export const BATAS_FOTO = 5_242_880;
export const BATAS_PDF = 10_485_760;
export const JENIS_DITERIMA = ["image/jpeg", "image/png", "application/pdf"] as const;
/** Atribut `accept` isian berkas: bantuan pemilih berkas peramban, bukan validasi. */
export const ACCEPT = ".jpg,.jpeg,.png,.pdf,image/jpeg,image/png,application/pdf";

/** Petunjuk berkas yang selalu tampil (Q12 A: PDF bertanda tangan digital ditolak, unggah hasil pindaian atau foto). */
export const PETUNJUK_BERKAS =
  "Foto JPG atau PNG paling besar 5 MiB, atau PDF paling besar 10 MiB. PDF bertanda tangan digital tidak diterima: unggah hasil pindaian atau foto dokumen yang sudah ditandatangani.";

/**
 * Pemeriksaan awal berkas di klien supaya HR tidak menunggu unggahan yang pasti ditolak. Jenis dari `File.type`
 * (pernyataan peramban, tidak dipercaya server) dan ukuran per jenis. `null` = boleh dicoba.
 */
export function periksaBerkas(berkas: { readonly type: string; readonly size: number } | null): string | null {
  if (berkas === null) return "Pilih berkas terlebih dahulu.";
  if (!(JENIS_DITERIMA as readonly string[]).includes(berkas.type)) {
    return "Jenis berkas tidak didukung. Unggah foto JPG atau PNG, atau dokumen PDF.";
  }
  if (berkas.size <= 0) return "Berkas kosong. Pilih berkas lain.";
  if (berkas.type === "application/pdf" && berkas.size > BATAS_PDF) {
    return "Ukuran berkas melebihi batas. PDF paling besar 10 MiB.";
  }
  if (berkas.type !== "application/pdf" && berkas.size > BATAS_FOTO) {
    return "Ukuran berkas melebihi batas. Foto JPG atau PNG paling besar 5 MiB.";
  }
  return null;
}

/** NIB/NPWP termasking (DG §6, brief S4): hanya empat digit terakhir, `************1234`. */
export function termasking(nilai: string): string {
  return `************${nilai.slice(-4)}`;
}

/** Waktu berlabel WIB (pola backoffice, catatan QA N5b): tidak bergantung zona peramban. */
export function formatWib(iso: string | null | undefined): string {
  if (iso === null || iso === undefined) return "-";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "-";
  const bagian = new Intl.DateTimeFormat("id-ID", {
    timeZone: "Asia/Jakarta",
    day: "numeric",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hourCycle: "h23",
  }).format(d);
  return `${bagian} WIB`;
}

/** Ukuran berkas untuk teks (MiB/KiB, koma desimal). */
export function formatUkuran(byte: number): string {
  if (byte >= 1_048_576) return `${(byte / 1_048_576).toFixed(1).replace(".", ",")} MiB`;
  return `${Math.max(1, Math.round(byte / 1024)).toString()} KiB`;
}
