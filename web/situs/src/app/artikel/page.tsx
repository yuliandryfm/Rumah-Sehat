import type { Metadata } from "next";
import { DaftarArtikel } from "../../komponen/Artikel.tsx";
import { ambilKlienKonten } from "../../konten/index.ts";
import { adalahUuid } from "../../konten/aturan-aman.ts";

export const metadata: Metadata = { title: "Artikel" };

/** Daftar artikel tayang; `?kursor=` hanya diteruskan bila berbentuk UUID (nilai lain diabaikan). */
export default async function HalamanArtikel({ searchParams }: { searchParams: Promise<Record<string, string | string[] | undefined>> }) {
  const { kursor } = await searchParams;
  const klien = ambilKlienKonten();
  const [pengantar, daftar] = await Promise.all([
    klien.ambilHalaman("articles"),
    klien.daftar({ jenis: "article", batas: 12, ...(typeof kursor === "string" && adalahUuid(kursor) ? { kursor } : {}) }),
  ]);
  return <DaftarArtikel pengantar={pengantar} artikel={daftar.item} kursorBerikut={daftar.kursorBerikut} />;
}
