/**
 * Menu member area HR (CR-039, keputusan user Q32 dan 5 menu "Ya", 2026-11-27): Beranda, Pesanan, Peserta,
 * Tagihan, Tim HR, dengan nama sehari-hari. Menu hanya kemudahan tampilan: otorisasi dan isolasi data
 * perusahaan tetap di API Rust (ADR-012, ADR-028), jadi menyembunyikan atau menampilkan menu bukan kontrol akses.
 * Item tanpa `href` (fitur belum ada) tampil abu-abu bertuliskan "segera hadir", bukan tautan dan tidak hilang.
 * `href` hanya boleh diisi bersama halamannya, supaya tidak ada tautan ke halaman yang belum ada.
 */
export interface ItemMenu {
  readonly label: string;
  /** Kosong = fitur belum ada ("segera hadir"). */
  readonly href?: string;
  /** Rute lain yang masih bagian dari item ini (menandai menu aktif); `href` sendiri sudah termasuk. */
  readonly cakup?: readonly string[];
}

export const MENU: readonly ItemMenu[] = [
  // Status verifikasi (/verifikasi) dicapai dari Beranda, jadi Beranda tetap menu aktif di sana.
  { label: "Beranda", href: "/", cakup: ["/verifikasi"] },
  { label: "Pesanan" },
  { label: "Peserta" },
  { label: "Tagihan" },
  // Rute tetap /kelola-hr; hanya label dan judul yang berganti dari "Kelola HR".
  { label: "Tim HR", href: "/kelola-hr" },
];

/** Hanya item yang punya halaman. */
export const MENU_TERSEDIA: readonly ItemMenu[] = MENU.filter((i) => i.href !== undefined);

/** Teks pengganti tautan pada item yang belum ada. */
export const LABEL_SEGERA_HADIR = "segera hadir";
