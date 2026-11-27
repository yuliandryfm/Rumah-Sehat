import { LABEL, MAKS, PETUNJUK, periksaSatu } from "../daftar/model.ts";
import type { ProfilPerusahaan } from "./klien.ts";

/**
 * Model formulir koreksi data perusahaan saat pengajuan Ditolak (CR-036, keputusan user Q20 = A; KEPUTUSAN-413).
 * Koreksi dikirim bersama pengajuan ulang (satu aksi, `POST /v1/corporate/company-corrections`). Validasi klien
 * sama dengan formulir daftar (`periksaSatu`: NIB 13 angka, NPWP 16 angka, NPWP 15 angka → petunjuk tambah 0);
 * server tetap sumber kebenaran. NIB dan NPWP tetap tersamar di tampilan biasa (Q23 = A): nilai tersimpan tidak
 * pernah dimasukkan ke isian, HR mengetik nilai lengkap yang baru hanya bila memilih "Ubah NIB"/"Ubah NPWP".
 * Isian hanya di memori halaman.
 */

export type FieldKoreksi = "legal_name" | "nib" | "npwp" | "office_address" | "office_phone";

/** Urutan tampil (dan urutan fokus ke galat pertama). */
export const FIELD_KOREKSI: readonly FieldKoreksi[] = ["legal_name", "nib", "npwp", "office_address", "office_phone"];

/** Field yang nilainya tersamar dan hanya diketik ulang bila HR memilih mengubahnya. */
export const FIELD_TERSAMAR: readonly FieldKoreksi[] = ["nib", "npwp"];

export const LABEL_KOREKSI: Record<FieldKoreksi, string> = {
  legal_name: LABEL.legal_name,
  nib: "NIB baru",
  npwp: "NPWP baru",
  office_address: LABEL.office_address,
  office_phone: LABEL.office_phone,
};

export const MAKS_KOREKSI: Record<FieldKoreksi, number> = {
  legal_name: MAKS.legal_name,
  nib: MAKS.nib,
  npwp: MAKS.npwp,
  office_address: MAKS.office_address,
  office_phone: MAKS.office_phone,
};

export const PETUNJUK_KOREKSI: Partial<Record<FieldKoreksi, string>> = {
  nib: PETUNJUK.nib ?? "",
  npwp: PETUNJUK.npwp ?? "",
};

export interface IsianKoreksi {
  legal_name: string;
  nib: string;
  npwp: string;
  office_address: string;
  office_phone: string;
}

/** Pilihan "Ubah NIB"/"Ubah NPWP". */
export type UbahTersamar = Readonly<Record<"nib" | "npwp", boolean>>;

export type GalatKoreksi = Partial<Record<FieldKoreksi, string>>;

/** Isian awal dari profil: nama, alamat, telepon terisi nilai tersimpan; NIB/NPWP kosong (tersamar, Q23 = A). */
export function isianAwal(p: ProfilPerusahaan): IsianKoreksi {
  return { legal_name: p.legal_name, nib: "", npwp: "", office_address: p.office_address, office_phone: p.office_phone };
}

/**
 * Badan permintaan koreksi: hanya field yang berubah (nama/alamat/telepon yang berbeda setelah di-trim, NIB/NPWP
 * yang dipilih untuk diubah). Objek kosong = tidak ada koreksi (pakai pengajuan ulang biasa).
 */
export function badanKoreksi(isian: IsianKoreksi, ubah: UbahTersamar, p: ProfilPerusahaan): Partial<IsianKoreksi> {
  const badan: Partial<IsianKoreksi> = {};
  for (const f of ["legal_name", "office_address", "office_phone"] as const) {
    const v = isian[f].trim();
    if (v !== p[f]) badan[f] = v;
  }
  for (const f of FIELD_TERSAMAR as readonly ("nib" | "npwp")[]) {
    if (ubah[f]) badan[f] = isian[f].trim();
  }
  return badan;
}

/** Galat klien untuk field yang akan dikirim; objek kosong = boleh dikirim. */
export function periksaKoreksi(isian: IsianKoreksi, ubah: UbahTersamar, p: ProfilPerusahaan): GalatKoreksi {
  const galat: GalatKoreksi = {};
  for (const [f, v] of Object.entries(badanKoreksi(isian, ubah, p)) as [FieldKoreksi, string][]) {
    const g = periksaSatu(f, v);
    if (g !== null) galat[f] = g;
  }
  return galat;
}

/** Galat 422 server (`errors[]`) dipetakan ke field koreksi; field lain diabaikan (pesan umum dipakai). */
export function galatServer(isian: Readonly<Record<string, string>>): GalatKoreksi {
  const g: GalatKoreksi = {};
  for (const [f, pesan] of Object.entries(isian)) {
    if ((FIELD_KOREKSI as readonly string[]).includes(f)) g[f as FieldKoreksi] = pesan;
  }
  return g;
}
