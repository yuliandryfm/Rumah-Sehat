import Link from "next/link";
import type { ReactNode } from "react";
import { NAV_KEBIJAKAN } from "../navigasi.ts";
import { NavUtama } from "./NavUtama.tsx";
import { TidakDiketahui } from "./TidakDiketahui.tsx";

/**
 * Tata letak situs: tautan lewati, header (logo, navigasi), `<main id="isi">`, footer. Mengikuti
 * docs/wireframes/s01-situs-publik.html. Menu seluler memakai `<details>` tanpa skrip.
 */
export function KerangkaSitus({ children }: { children: ReactNode }) {
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
          </ul>
        </nav>
        <div className="situs-kecil">Dioperasikan oleh PT. Arunika</div>
        <p className="situs-legal">
          [Hak cipta dan alamat badan usaha] <TidakDiketahui />
        </p>
      </footer>
    </>
  );
}
