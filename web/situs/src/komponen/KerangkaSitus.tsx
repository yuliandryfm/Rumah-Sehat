import Link from "next/link";
import type { ReactNode } from "react";
import { NAV_KEBIJAKAN } from "../navigasi.ts";
import { ambilUrlKorporat } from "../konten/tautan-korporat.ts";
import { BandUnduh } from "./BandUnduh.tsx";
import { Logo } from "./Logo.tsx";
import { NavUtama } from "./NavUtama.tsx";
import { TautanToko } from "./TautanToko.tsx";

/**
 * Tata letak situs: tautan lewati, header (logo, navigasi), `<main id="isi">`, footer. Mengikuti
 * docs/wireframes/s01-situs-publik.html. Navigasi berupa deretan tautan di header tanpa menu lipat (CR-038); tautan toko aplikasi di pojok kanan.
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
        <div className="situs-wadah situs-top__isi">
          <Link className="situs-logo" href="/">
            <Logo />
            <span>Ruang Sehat</span>
          </Link>
          <div className="situs-nav-baris">
            <NavUtama label="Utama" />
          </div>
          <TautanToko />
        </div>
      </header>
      <main id="isi">
        {children}
        <BandUnduh />
      </main>
      <footer className="situs-foot">
        <div className="situs-wadah situs-foot__grid">
          <div className="situs-foot__merek">
            <p className="situs-foot__nama">
              <Logo kelas="situs-logo__tanda situs-logo__tanda--kaki" />
              <span>Ruang Sehat</span>
            </p>
            <p className="situs-kecil">Ruang Sehat berperan sebagai platform yang menghubungkan pasien dengan rekanan.</p>
            <p className="situs-kecil situs-foot__redup">Dioperasikan oleh PT. Arunika</p>
          </div>
          <nav className="situs-foot__nav" aria-label="Tautan kaki">
            <div>
              <p className="situs-foot__judul" id="kaki-layanan">
                Layanan
              </p>
              <ul aria-labelledby="kaki-layanan">
                <li>
                  <Link href="/layanan" prefetch={false}>
                    Semua layanan
                  </Link>
                </li>
                <li>
                  <Link href="/unduh" prefetch={false}>
                    Unduh aplikasi
                  </Link>
                </li>
                <li>
                  <Link href="/artikel" prefetch={false}>
                    Artikel
                  </Link>
                </li>
              </ul>
            </div>
            <div>
              <p className="situs-foot__judul" id="kaki-perusahaan">
                Perusahaan
              </p>
              <ul aria-labelledby="kaki-perusahaan">
                <li>
                  <Link href="/untuk-perusahaan" prefetch={false}>
                    Untuk Perusahaan
                  </Link>
                </li>
                {urlKorporat === null ? null : (
                  <li>
                    <a href={urlKorporat} rel="noopener noreferrer">
                      Masuk HR
                    </a>
                  </li>
                )}
                <li>
                  <Link href="/rekanan" prefetch={false}>
                    Rekanan
                  </Link>
                </li>
              </ul>
            </div>
            <div>
              <p className="situs-foot__judul" id="kaki-bantuan">
                Bantuan
              </p>
              <ul aria-labelledby="kaki-bantuan">
                <li>
                  <Link href="/tentang-kami" prefetch={false}>
                    Tentang Kami
                  </Link>
                </li>
                <li>
                  <Link href="/kontak" prefetch={false}>
                    Kontak
                  </Link>
                </li>
                <li>
                  <Link href={NAV_KEBIJAKAN.href} prefetch={false}>
                    {NAV_KEBIJAKAN.label}
                  </Link>
                </li>
              </ul>
            </div>
          </nav>
          <p className="situs-legal">
            {/* Teks hak cipta: keputusan user 2026-11-26 (opsi A, tanpa alamat badan usaha). */}© 2026 PT. Arunika. Hak cipta dilindungi.
          </p>
        </div>
      </footer>
    </>
  );
}
