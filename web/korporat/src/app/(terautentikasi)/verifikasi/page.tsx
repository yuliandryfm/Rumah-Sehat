import type { Metadata } from "next";
import { StatusVerifikasi } from "../../../komponen/verifikasi/StatusVerifikasi.tsx";
import { wajibSesi } from "../../../lib/server/sesi.ts";

export const metadata: Metadata = { title: "Status verifikasi" };

/** Status verifikasi dan dokumen perusahaan (F1-T103, wireframe K01 layar 7-8, US-1901 AC6, US-1902 AC3). */
export default async function HalamanVerifikasi() {
  const { csrfToken } = await wajibSesi();
  return <StatusVerifikasi csrfToken={csrfToken} />;
}
