import type { Metadata } from "next";
import { HalamanKonten } from "../../komponen/HalamanKonten.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Kontak" };

export default async function HalamanKontak() {
  return <HalamanKonten konten={await ambilHalamanTayang("contact")} susunan="kartu" />;
}
