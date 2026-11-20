import { Beranda } from "../komponen/Beranda.tsx";
import { ambilKlienKonten } from "../konten/index.ts";

export default async function HalamanBeranda() {
  // Galat (KontenTidakTersedia atau lainnya) ditangkap `error.tsx`: halaman galat umum, tanpa rincian.
  const konten = await ambilKlienKonten().ambilBeranda();
  return <Beranda konten={konten} />;
}
