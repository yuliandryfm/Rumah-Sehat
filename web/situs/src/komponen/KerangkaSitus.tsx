import Link from "next/link";
import type { ReactNode } from "react";
import { NAV_KEBIJAKAN } from "../navigasi.ts";
import { ambilUrlKorporat } from "../konten/tautan-korporat.ts";
import { NavUtama } from "./NavUtama.tsx";

/**
 * Tata letak situs: tautan lewati, header (logo, navigasi), `<main id="isi">`, footer. Mengikuti
 * docs/wireframes/s01-situs-publik.html. Menu seluler memakai `<details>` tanpa skrip.
 */
export function KerangkaSitus({ children }: { children: ReactNode }) {
  // Dibaca per permintaan (layout dinamis lewat `connection()`); tidak valid → tautan tidak tampil (CR-032).
  const urlKorporat = ambilUrlKorporat();
  return (
    <>
      <a className="situs-skip" href="#isi">
        Lewati ke isi
      </a>
      <header className="situs-top">
        <Link className="situs-logo" href="/">
          Ruang Sehat
        </Link>
        <span className="situs-grow" />
        <div className="situs-nav-lebar">
          <NavUtama label="Utama" />
        </div>
        <details className="situs-menu">
          <summary aria-label="Menu utama">Menu</summary>
          <NavUtama label="Utama (seluler)" />
        </details>
      </header>
      <main id="isi">{children}</main>
      <footer className="situs-foot">
        <div>
          <b>Ruang Sehat</b>
          <p className="situs-kecil">Ruang Sehat berperan sebagai platform yang menghubungkan pasien dengan rekanan.</p>
        </div>
        <nav aria-label="Tautan kaki">
          <ul>
            <li>
              <Link href={NAV_KEBIJAKAN.href} prefetch={false}>
                {NAV_KEBIJAKAN.label}
              </Link>
            </li>
            <li>
              <Link href="/kontak" prefetch={false}>
                Kontak
              </Link>
            </li>
            {urlKorporat === null ? null : (
              <li>
                <a href={urlKorporat} rel="noopener noreferrer">
                  Masuk HR
                </a>
              </li>
            )}
          </ul>
        </nav>
        <div className="situs-kecil">Dioperasikan oleh PT. Arunika</div>
        <p className="situs-legal">
          {/* Teks hak cipta: keputusan user 2026-11-26 (opsi A, tanpa alamat badan usaha). */}
          © 2026 PT. Arunika. Hak cipta dilindungi.
        </p>
      </footer>
    </>
  );
}
