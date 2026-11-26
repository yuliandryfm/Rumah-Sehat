import type { Metadata } from "next";
import Link from "next/link";
import { IkonLingkar } from "../komponen/Ikon.tsx";

export const metadata: Metadata = { title: "Halaman tidak ditemukan" };

export default function TidakDitemukan() {
  return (
    <div className="situs-status">
      <IkonLingkar nama="generik" />
      <h1>Halaman tidak ditemukan</h1>
      <p>Halaman yang Anda cari tidak tersedia.</p>
      <Link className="rs-btn rs-btn--primary" href="/">
        Ke Beranda
      </Link>
    </div>
  );
}
