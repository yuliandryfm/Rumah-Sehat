import type { Metadata } from "next";
import { HalamanKonten } from "../../komponen/HalamanKonten.tsx";
import { MasukMemberArea } from "../../komponen/MasukMemberArea.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Untuk Perusahaan" };

export default async function HalamanUntukPerusahaan() {
  const konten = await ambilHalamanTayang("for_business");
  return (
    <>
      <HalamanKonten konten={konten} />
      <MasukMemberArea />
    </>
  );
}
