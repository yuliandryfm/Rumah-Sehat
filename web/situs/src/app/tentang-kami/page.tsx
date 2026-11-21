import type { Metadata } from "next";
import { HalamanKonten } from "../../komponen/HalamanKonten.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Tentang Kami" };

export default async function HalamanTentangKami() {
  return <HalamanKonten konten={await ambilHalamanTayang("about")} />;
}
