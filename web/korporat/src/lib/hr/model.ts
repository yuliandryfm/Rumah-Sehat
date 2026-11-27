import type { BadgeTone } from "@rs/ui";
import type { AnggotaHr, StatusHr } from "./klien.ts";

/** Label dan nada status akun HR (wireframe K01 layar 10). */
export const LABEL_STATUS: Record<StatusHr, { readonly label: string; readonly tone: BadgeTone }> = {
  active: { label: "Aktif", tone: "success" },
  pending_activation: { label: "Menunggu aktivasi", tone: "warning" },
  invitation_processing: { label: "Undangan diproses", tone: "warning" },
  inactive: { label: "Nonaktif", tone: "neutral" },
};

/** HR dapat dinonaktifkan dari UI bila bukan diri sendiri dan belum nonaktif (server tetap memvalidasi). */
export function dapatDinonaktifkan(a: AnggotaHr): boolean {
  // Undangan yang masih diproses belum dapat dinonaktifkan (server menolak 409).
  return !a.is_self && a.status !== "inactive" && a.status !== "invitation_processing";
}

export function namaTampil(a: AnggotaHr): string {
  const nama = a.full_name?.trim();
  return nama === undefined || nama === "" ? a.email : nama;
}

/**
 * Pesan tunggal setelah undangan diterima (202). Server menjawab sama untuk email baru dan email yang sudah
 * terdaftar (anti-enumerasi, KEPUTUSAN-306): UI tidak boleh membedakan keduanya dan tidak menyebut email.
 */
export const PESAN_UNDANGAN_DIPROSES =
  "Undangan sedang diproses. Bila email valid, penerima akan menerima tautan aktivasi yang berlaku 24 jam.";

/** Batas panjang email di kontrak (`maxLength` 254). */
export const EMAIL_MAKS = 254;

/** Bantuan di klien saja: bentuk dasar `x@y.z` tanpa spasi. Server tetap penentu validitas (422). */
export function emailTampakValid(email: string): boolean {
  const e = email.trim();
  return e.length > 0 && e.length <= EMAIL_MAKS && /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(e);
}
