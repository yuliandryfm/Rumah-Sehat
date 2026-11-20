import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = { title: "Halaman tidak ditemukan" };

export default function TidakDitemukan() {
  return (
    <div className="situs-status">
      <h1>Halaman tidak ditemukan</h1>
      <p>Halaman yang Anda cari tidak tersedia.</p>
      <Link className="rs-btn rs-btn--primary" href="/">
        Ke Beranda
      </Link>
    </div>
  );
}
