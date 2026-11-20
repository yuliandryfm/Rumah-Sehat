import type { Metadata } from "next";
import Link from "next/link";
import { LayarStatus } from "../../../komponen/LayarStatus.tsx";

export const metadata: Metadata = { title: "Masuk tidak berhasil" };

/** Tujuan BFF saat login gagal/kedaluwarsa (`loginErrorPath`); pesan umum, tanpa rincian penyebab. */
export default async function MasukGagal({ searchParams }: { searchParams: Promise<{ auth_error?: string }> }) {
  const { auth_error: sebab } = await searchParams;
  return (
    <LayarStatus judul="Masuk tidak berhasil">
      <p>
        {sebab === "expired"
          ? "Proses masuk kedaluwarsa. Silakan masuk lagi."
          : "Masuk belum dapat diselesaikan. Silakan coba lagi."}
      </p>
      <a className="rs-btn rs-btn--primary" href="/auth/login">
        Masuk lagi
      </a>
      <Link href="/akun-nonaktif" prefetch={false}>
        Akun saya nonaktif
      </Link>
    </LayarStatus>
  );
}
