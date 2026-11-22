import type { BackofficeComponents } from "@rs/api-client";

type S = BackofficeComponents["schemas"];
export type ItemAntrean = S["QueueItemDto"];
export type Detail = S["VerificationDetailDto"];
export type AlasanTolak = S["RejectionReasonDto"];
export type FotoDokumen = S["DocumentPhotoDto"];
export type StatusPengajuan = S["RequestStatusDto"];

export const LABEL_DOKUMEN: Record<string, string> = { ktp: "KTP", kk: "KK" };
export const LABEL_STATUS: Record<StatusPengajuan, string> = {
  pending: "Menunggu verifikasi",
  approved: "Disetujui",
  rejected: "Ditolak",
  cancelled: "Dibatalkan pasien",
};
export const LABEL_SEX: Record<string, string> = { male: "Laki-laki", female: "Perempuan" };

/** Kode alasan untuk NIK terpakai (US-201 AC7); daftar alasan tetap dari API, kode ini hanya untuk menyorot pilihan. */
export const KODE_NIK_TIDAK_TERSEDIA = "nik_unavailable";
/** Kode alasan yang mewajibkan keterangan (US-201 AC3a). */
export const KODE_LAINNYA = "other";
export const BATAS_KETERANGAN = 500;

/** NIK di daftar: hanya empat digit terakhir (wireframe C02 `************0003`). */
export function nikTermasking(last4: string): string {
  return `************${last4}`;
}

/** Waktu selalu berlabel WIB (catatan QA N5b), format tetap supaya tidak bergantung zona peramban. */
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

export function formatTanggal(iso: string): string {
  const d = new Date(`${iso}T00:00:00Z`);
  if (Number.isNaN(d.getTime())) return "-";
  return new Intl.DateTimeFormat("id-ID", { timeZone: "UTC", day: "numeric", month: "short", year: "numeric" }).format(d);
}
