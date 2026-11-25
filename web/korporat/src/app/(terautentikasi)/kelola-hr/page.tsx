import type { Metadata } from "next";
import { KelolaHr } from "../../../komponen/hr/KelolaHr.tsx";
import { wajibSesi } from "../../../lib/server/sesi.ts";

export const metadata: Metadata = { title: "Kelola HR" };

/** Kelola HR (F1-T108, wireframe K01 layar 10, US-1904 AC2). */
export default async function HalamanKelolaHr() {
  const { csrfToken } = await wajibSesi();
  return <KelolaHr csrfToken={csrfToken} />;
}
