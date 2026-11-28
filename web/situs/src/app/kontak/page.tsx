import type { Metadata } from "next";
import { BannerKeamanan } from "../../komponen/BannerKeamanan.tsx";
import { HalamanKonten } from "../../komponen/HalamanKonten.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Kontak" };

export default async function HalamanKontak() {
  const konten = await ambilHalamanTayang("contact");
  return (
    <>
      <HalamanKonten konten={konten} susunan="kartu" />
      <BannerKeamanan />
    </>
  );
}
