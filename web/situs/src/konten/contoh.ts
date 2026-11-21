import type { HalamanSitus, HasilDaftar, KlienKonten, Konten } from "./tipe.ts";

const JUDUL_CONTOH: Readonly<Record<HalamanSitus, string>> = {
  home: "Beranda",
  about: "Tentang Kami",
  services: "Layanan",
  for_business: "Untuk Perusahaan",
  partners: "Rekanan",
  articles: "Artikel",
  contact: "Kontak",
  download_app: "Unduh aplikasi",
  privacy_terms: "Kebijakan privasi & syarat",
};

/**
 * Klien contoh untuk test dan pratinjau lokal. Bukan konten bisnis: judul hanya label navigasi, isi hanya
 * kalimat penanda bahwa ini contoh; Beranda dan Artikel kosong supaya penanda `[TIDAK DIKETAHUI]` tampil.
 * Tidak pernah dipakai di produksi (lihat `index.ts`).
 */
export class KlienKontenContoh implements KlienKonten {
  ambilHalaman(halaman: HalamanSitus): Promise<Konten | null> {
    if (halaman === "home" || halaman === "articles") return Promise.resolve(null);
    return Promise.resolve({
      content_id: "00000000-0000-7000-8000-000000000000",
      content_type: "info_page",
      title: JUDUL_CONTOH[halaman],
      body: "Isi contoh untuk pratinjau lokal. Bukan konten bisnis.",
      published_at: "2026-01-01T00:00:00Z",
      site_page: halaman,
    });
  }

  ambilItem(): Promise<Konten | null> {
    return Promise.resolve(null);
  }

  daftar(): Promise<HasilDaftar> {
    return Promise.resolve({ item: [], kursorBerikut: null });
  }
}
