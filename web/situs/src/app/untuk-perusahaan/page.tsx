import type { Metadata } from "next";
import { HalamanKonten } from "../../komponen/HalamanKonten.tsx";
import { MasukMemberArea } from "../../komponen/MasukMemberArea.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Untuk Perusahaan" };

/** Hero gelap ala konsep B untuk audiens HR (keputusan user Q15, CR-033). */
export default async function HalamanUntukPerusahaan() {
  const konten = await ambilHalamanTayang("for_business");
  return (
    <>
      <HalamanKonten konten={konten} gelap label="Untuk HR dan perusahaan" />
      <MasukMemberArea />
    </>
  );
}
