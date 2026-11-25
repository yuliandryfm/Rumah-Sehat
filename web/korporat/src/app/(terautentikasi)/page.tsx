import type { Metadata } from "next";
import { DasborHr } from "../../komponen/hr/DasborHr.tsx";
import { wajibSesi } from "../../lib/server/sesi.ts";

export const metadata: Metadata = { title: "Dasbor" };

/** Dasbor HR (F1-T108, wireframe K01 layar 9). Data dimuat dari klien lewat BFF; otorisasi di API Rust. */
export default async function Dasbor() {
  const { csrfToken } = await wajibSesi();
  return <DasborHr csrfToken={csrfToken} />;
}
