"use client";

import { Button } from "@rs/ui";
import Link from "next/link";

/** Galat umum: tidak menampilkan pesan, jejak, atau digest (tidak membocorkan rincian internal). */
export default function Galat({ reset }: { error: Error & { digest?: string }; reset: () => void }) {
  return (
    <div className="situs-status" role="alert">
      <h1>Terjadi kesalahan</h1>
      <p>Halaman belum dapat ditampilkan. Silakan coba lagi beberapa saat lagi.</p>
      <Button onClick={reset}>Coba lagi</Button>
      <Link href="/">Ke Beranda</Link>
    </div>
  );
}
