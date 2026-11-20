import type { KlienKonten, KontenBeranda } from "./tipe.ts";

/**
 * Data contoh untuk test dan pratinjau lokal. Bukan konten bisnis: nama enam layanan dan uraian
 * singkatnya diambil dari wireframe `docs/wireframes/s01-situs-publik.html` (UP-63); teks lain null
 * (`[TIDAK DIKETAHUI]`). Tidak pernah dipakai di produksi (lihat `index.ts`).
 */
export const BERANDA_CONTOH: KontenBeranda = {
  judulUtama: null,
  kalimatPembuka: null,
  layanan: [
    { judul: "Konsultasi Online", uraian: "Chat dan video dengan dokter atau psikolog." },
    { judul: "Pendaftaran Klinik", uraian: "Daftar ke klinik rekanan." },
    { judul: "Pendaftaran Lab", uraian: "Daftar ke lab rekanan." },
    { judul: "Home Visit", uraian: "Tenaga medis datang ke rumah." },
    { judul: "Biaya MCU & Lab", uraian: "Informasi biaya pemeriksaan." },
    { judul: "Belanja Produk", uraian: "Obat bebas, kecantikan, dan produk kesehatan." },
  ],
  ringkasanKorporat: null,
  artikelTerbaru: [],
};

export class KlienKontenContoh implements KlienKonten {
  ambilBeranda(): Promise<KontenBeranda> {
    return Promise.resolve(BERANDA_CONTOH);
  }
}
