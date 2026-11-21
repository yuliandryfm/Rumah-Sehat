import type { Metadata } from "next";
import { HalamanKonten } from "../../komponen/HalamanKonten.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Layanan" };

export default async function HalamanLayanan() {
  return <HalamanKonten konten={await ambilHalamanTayang("services")} />;
}
