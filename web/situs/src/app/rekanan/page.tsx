import type { Metadata } from "next";
import { HalamanKonten } from "../../komponen/HalamanKonten.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Rekanan" };

export default async function HalamanRekanan() {
  return <HalamanKonten konten={await ambilHalamanTayang("partners")} />;
}
