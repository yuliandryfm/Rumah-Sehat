import { notFound } from "next/navigation";
import { DetailArtikel } from "../../../komponen/Artikel.tsx";
import { adalahUuid } from "../../../konten/aturan-aman.ts";
import { ambilKlienKonten } from "../../../konten/index.ts";

/**
 * Satu artikel tayang. ID bukan UUID, konten tidak tayang, atau bukan artikel menjadi halaman tidak
 * ditemukan tanpa membedakan sebabnya (US-1801 AC3).
 */
export default async function HalamanDetailArtikel({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  if (!adalahUuid(id)) notFound();
  const artikel = await ambilKlienKonten().ambilItem(id);
  if (artikel?.content_type !== "article") notFound();
  return <DetailArtikel artikel={artikel} />;
}
