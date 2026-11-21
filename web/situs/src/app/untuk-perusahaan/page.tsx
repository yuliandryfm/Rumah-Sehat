import type { Metadata } from "next";
import { HalamanKonten } from "../../komponen/HalamanKonten.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Untuk Perusahaan" };

export default async function HalamanUntukPerusahaan() {
  return <HalamanKonten konten={await ambilHalamanTayang("for_business")} />;
}
