import type { Metadata } from "next";
import { HalamanLayanan } from "../../komponen/Layanan.tsx";
import { ambilHalamanTayang } from "../../konten/halaman.ts";

export const metadata: Metadata = { title: "Layanan" };

export default async function HalamanLayananRute() {
  return <HalamanLayanan konten={await ambilHalamanTayang("services")} />;
}
