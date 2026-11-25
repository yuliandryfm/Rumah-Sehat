import type { BadgeTone } from "@rs/ui";
import type { AnggotaHr, StatusHr } from "./klien.ts";

/** Label dan nada status akun HR (wireframe K01 layar 10). */
export const LABEL_STATUS: Record<StatusHr, { readonly label: string; readonly tone: BadgeTone }> = {
  active: { label: "Aktif", tone: "success" },
  pending_activation: { label: "Menunggu aktivasi", tone: "warning" },
  inactive: { label: "Nonaktif", tone: "neutral" },
};

/** Ringkasan untuk dasbor: jumlah HR aktif dan undangan yang belum diaktivasi. */
export function ringkas(anggota: readonly AnggotaHr[]): { readonly aktif: number; readonly menunggu: number } {
  return {
    aktif: anggota.filter((a) => a.status === "active").length,
    menunggu: anggota.filter((a) => a.status === "pending_activation").length,
  };
}

/** HR dapat dinonaktifkan dari UI bila bukan diri sendiri dan belum nonaktif (server tetap memvalidasi). */
export function dapatDinonaktifkan(a: AnggotaHr): boolean {
  return !a.is_self && a.status !== "inactive";
}

export function namaTampil(a: AnggotaHr): string {
  const nama = a.full_name?.trim();
  return nama === undefined || nama === "" ? a.email : nama;
}
