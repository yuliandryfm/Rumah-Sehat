import type { Metadata } from "next";
import { Beranda } from "../../komponen/beranda/Beranda.tsx";
import { wajibSesi } from "../../lib/server/sesi.ts";

export const metadata: Metadata = { title: "Beranda" };

/** Beranda HR (CR-039, Q32): status perusahaan dan satu langkah berikutnya. Data dimuat lewat BFF; otorisasi di API Rust. */
export default async function HalamanBeranda() {
  const { csrfToken } = await wajibSesi();
  return <Beranda csrfToken={csrfToken} />;
}
