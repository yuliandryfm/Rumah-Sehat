import type { Metadata } from "next";
import { DataTidakDitemukan } from "../komponen/DataTidakDitemukan.tsx";
import { korelasiPermintaan } from "../lib/server/korelasi.ts";

export const metadata: Metadata = { title: "Data tidak ditemukan" };

export default async function TidakDitemukan() {
  return <DataTidakDitemukan kode={await korelasiPermintaan()} />;
}
