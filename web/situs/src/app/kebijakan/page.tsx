import type { Metadata } from "next";
import { HalamanKonten } from "../../komponen/HalamanKonten.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Kebijakan privasi & syarat (draf)" };

/** Teks final menunggu keputusan hukum: halaman selalu bertanda draf (US-1801 AC4). */
export default async function HalamanKebijakan() {
  return <HalamanKonten konten={await ambilHalamanTayang("privacy_terms")} draf />;
}
