/**
 * Navigasi member area (wireframe K01 layar 9). Menu hanya kemudahan tampilan: otorisasi dan isolasi data
 * perusahaan tetap di API Rust (ADR-012, ADR-028), jadi menyembunyikan menu bukan kontrol akses.
 * Menu layanan korporat tampil nonaktif dengan alasan tertulis "belum tersedia" (bukan hilang). Isinya
 * ditambahkan task fiturnya bersama halamannya (Kelola HR: F1-T108; Profil perusahaan menunggu endpoint profil), supaya tidak ada tautan ke
 * halaman yang belum ada.
 */
export interface ItemNav {
  readonly href: string;
  readonly label: string;
}

/** Menu aktif. Hanya halaman yang sudah ada. */
export const ITEM_NAV: readonly ItemNav[] = [
  { href: "/", label: "Dasbor" },
  { href: "/verifikasi", label: "Status verifikasi" },
  { href: "/kelola-hr", label: "Kelola HR" },
];

/** Menu layanan korporat yang belum tersedia (K01 layar 9): teks dan alasan tertulis, bukan tautan. */
export const ITEM_BELUM_TERSEDIA: readonly string[] = ["Pesanan", "Peserta", "Invoice"];
