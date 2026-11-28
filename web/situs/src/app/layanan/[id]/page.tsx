import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { DetailLayanan } from "../../../komponen/Layanan.tsx";
import { ambilHalamanTayang } from "../../../konten/halaman.ts";
import { uraiLayanan } from "../../../konten/struktur.ts";

export const metadata: Metadata = { title: "Layanan" };

/**
 * Detail satu layanan dari kartu `##` halaman `services` (ID `layanan-N` = urutan di CMS). ID tak dikenal atau
 * halaman tidak tayang menjadi halaman tidak ditemukan tanpa membedakan sebabnya (US-1801 AC3).
 */
export default async function HalamanDetailLayanan({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  if (!/^layanan-\d{1,3}$/.test(id)) notFound();
  const kartu = uraiLayanan((await ambilHalamanTayang("services")).body)?.kartu.find((k) => k.id === id);
  if (kartu === undefined) notFound();
  return <DetailLayanan kartu={kartu} />;
}
