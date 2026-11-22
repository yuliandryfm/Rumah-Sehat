import type { Metadata } from "next";
import { HalamanKonten } from "../../komponen/HalamanKonten.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Unduh aplikasi" };

export default async function HalamanUnduh() {
  return <HalamanKonten konten={await ambilHalamanTayang("download_app")} />;
}
