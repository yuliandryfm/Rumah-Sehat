import type { Metadata } from "next";
import { HalamanKonten } from "../../komponen/HalamanKonten.tsx";
import { MasukMemberArea } from "../../komponen/MasukMemberArea.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Untuk Perusahaan" };

/** Pita judul terang seperti halaman lain (CR-042: desain baru menggantikan hero gelap Q15/CR-033; seksi korporat gelap tetap di Beranda). */
export default async function HalamanUntukPerusahaan() {
  const konten = await ambilHalamanTayang("for_business");
  return (
    <>
      <HalamanKonten konten={konten} label="Untuk HR dan perusahaan" />
      <MasukMemberArea />
    </>
  );
}
