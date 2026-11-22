/**
 * Navigasi member area (wireframe K01 layar 9). Menu hanya kemudahan tampilan: otorisasi dan isolasi data
 * perusahaan tetap di API Rust (ADR-012, ADR-028), jadi menyembunyikan menu bukan kontrol akses.
 * Menu layanan korporat tampil nonaktif dengan alasan tertulis "belum tersedia" (bukan hilang). Isinya
 * ditambahkan task fiturnya (profil, kelola HR: F1-T108) bersama halamannya, supaya tidak ada tautan ke
 * halaman yang belum ada.
 */
export interface ItemNav {
  readonly href: string;
  readonly label: string;
}

/** Menu aktif. Hanya halaman yang sudah ada. */
export const ITEM_NAV: readonly ItemNav[] = [{ href: "/", label: "Dasbor" }];

/** Menu layanan korporat yang belum tersedia (K01 layar 9): teks dan alasan tertulis, bukan tautan. */
export const ITEM_BELUM_TERSEDIA: readonly string[] = ["Pesanan", "Peserta", "Invoice"];
