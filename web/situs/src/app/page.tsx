import { Beranda } from "../komponen/Beranda.tsx";
import { ambilKlienKonten } from "../konten/index.ts";

export default async function HalamanBeranda() {
  // Galat (KontenTidakTersedia atau lainnya) ditangkap `error.tsx`: halaman galat umum, tanpa rincian.
  const klien = ambilKlienKonten();
  const [beranda, layanan, untukPerusahaan, artikel] = await Promise.all([
    klien.ambilHalaman("home"),
    klien.ambilHalaman("services"),
    klien.ambilHalaman("for_business"),
    klien.daftar({ jenis: "article", batas: 3 }),
  ]);
  return <Beranda data={{ beranda, layanan, untukPerusahaan, artikelTerbaru: artikel.item }} />;
}
