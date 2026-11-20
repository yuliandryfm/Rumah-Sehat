/**
 * Sembilan halaman situs (docs/wireframes/s01-situs-publik.html; US-1801). Path adalah pilihan
 * implementasi awal; halaman-halamannya dibuat di F1-T96 dan F1-T97 (sampai itu tautan ke halaman
 * yang belum ada menampilkan halaman tidak ditemukan).
 */
export interface ItemNavigasi {
  href: string;
  label: string;
}

export const NAV_UTAMA: readonly ItemNavigasi[] = [
  { href: "/", label: "Beranda" },
  { href: "/tentang-kami", label: "Tentang Kami" },
  { href: "/layanan", label: "Layanan" },
  { href: "/untuk-perusahaan", label: "Untuk Perusahaan" },
  { href: "/rekanan", label: "Rekanan" },
  { href: "/artikel", label: "Artikel" },
  { href: "/kontak", label: "Kontak" },
];

/** Tombol ajakan di navigasi (halaman kedelapan). */
export const NAV_UNDUH: ItemNavigasi = { href: "/unduh", label: "Unduh aplikasi" };

/** Halaman kesembilan, ditautkan dari footer. */
export const NAV_KEBIJAKAN: ItemNavigasi = { href: "/kebijakan", label: "Kebijakan privasi & syarat" };

export const SEMBILAN_HALAMAN: readonly ItemNavigasi[] = [...NAV_UTAMA, NAV_UNDUH, NAV_KEBIJAKAN];
