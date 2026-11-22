/**
 * Navigasi backoffice menurut sub-peran admin Ruang Sehat (PRD §3.1 `[UP-18]`, ADR-012 butir 1).
 * Menu hanya kemudahan tampilan: otorisasi data tetap di API Rust (ADR-012), jadi menyembunyikan menu
 * bukan kontrol akses.
 */
export const SUB_PERAN = [
  "Super Admin",
  "Verifikator",
  "Layanan Pelanggan",
  "Keuangan",
  "Konten",
  "Auditor",
  "Petugas PDP",
] as const;

export type SubPeran = (typeof SUB_PERAN)[number];

export interface ItemNav {
  readonly href: string;
  readonly label: string;
  /** Sub-peran yang melihat menu ini; `"semua"` untuk setiap admin yang sudah masuk. */
  readonly untuk: "semua" | readonly SubPeran[];
}

/**
 * Daftar menu (Beranda dan Konten/CMS, F1-T95): menu fitur (misalnya antrean Verifikator, F1-T64) ditambahkan oleh task
 * fiturnya bersama halamannya, supaya tidak ada tautan ke halaman yang belum ada.
 */
export const ITEM_NAV: readonly ItemNav[] = [
  { href: "/", label: "Beranda", untuk: "semua" },
  { href: "/konten", label: "Konten", untuk: ["Konten"] },
  { href: "/verifikasi", label: "Verifikasi identitas", untuk: ["Verifikator"] },
];

/** Menu yang tampil untuk sub-peran tertentu; tanpa sub-peran yang dikenal hanya menu `"semua"`. */
export function menuUntuk(subPeran: readonly string[], daftar: readonly ItemNav[] = ITEM_NAV): readonly ItemNav[] {
  return daftar.filter((i) => i.untuk === "semua" || i.untuk.some((s) => subPeran.includes(s)));
}
