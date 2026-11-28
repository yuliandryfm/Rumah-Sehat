import type { ReactNode } from "react";
import type { NamaIkonLayanan } from "../konten/struktur.ts";

/**
 * Ikon garis situs (CR-033 C-02, KEPUTUSAN-355): SVG sebaris buatan sendiri, kisi 24, garis 1,75, ujung
 * membulat, warna `currentColor` (warna dari kelas CSS, bukan atribut). Tanpa set ikon pihak ketiga, tanpa
 * berkas eksternal. Selalu dekoratif (`aria-hidden`): makna dibawa teks di sebelahnya (DG §3.5).
 */
export type NamaIkon = NamaIkonLayanan | "cek" | "panah" | "video" | "struk" | "unduh" | "perisai";

const JALUR: Readonly<Record<NamaIkon, ReactNode>> = {
  konsultasi: (
    <>
      <path d="M5 4h14a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-8l-5 4v-4H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z" />
      <path d="M12 7.5v6M9 10.5h6" />
    </>
  ),
  psikolog: (
    <>
      <path d="M5 4h14a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-8l-5 4v-4H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z" />
      <path d="M12 14s-3.5-2-3.5-4.4A1.9 1.9 0 0 1 12 8.6a1.9 1.9 0 0 1 3.5 1C15.5 12 12 14 12 14z" />
    </>
  ),
  klinik: (
    <>
      <path d="M4 21V8.5L12 3l8 5.5V21" />
      <path d="M2.5 21h19" />
      <path d="M12 7.5v5M9.5 10h5" />
      <path d="M10 21v-4.5h4V21" />
    </>
  ),
  lab: (
    <>
      <path d="M8.5 3h7" />
      <path d="M10 3v6.2L4.9 18a2 2 0 0 0 1.7 3h10.8a2 2 0 0 0 1.7-3L14 9.2V3" />
      <path d="M7.4 14.5h9.2" />
    </>
  ),
  "home-visit": (
    <>
      <path d="M3 11.5 12 4l9 7.5" />
      <path d="M5.5 9.5V20h13V9.5" />
      <path d="M12 11.5v5M9.5 14h5" />
    </>
  ),
  biaya: (
    <>
      <path d="M9 3.5h6a1 1 0 0 1 1 1V6H8V4.5a1 1 0 0 1 1-1z" />
      <path d="M8 5H6.5A1.5 1.5 0 0 0 5 6.5v13A1.5 1.5 0 0 0 6.5 21h11a1.5 1.5 0 0 0 1.5-1.5v-13A1.5 1.5 0 0 0 17.5 5H16" />
      <path d="M8.5 11h7M8.5 14.5h7M8.5 18h4" />
    </>
  ),
  belanja: (
    <>
      <path d="M5 8h14l-1.1 12.1a1 1 0 0 1-1 .9H7.1a1 1 0 0 1-1-.9L5 8z" />
      <path d="M9 10V6.5a3 3 0 0 1 6 0V10" />
    </>
  ),
  korporat: (
    <>
      <rect x="3" y="7" width="18" height="13" rx="2" />
      <path d="M9 7V5.5A1.5 1.5 0 0 1 10.5 4h3A1.5 1.5 0 0 1 15 5.5V7" />
      <path d="M3 12.5h18" />
      <path d="M11 12.5V14h2v-1.5" />
    </>
  ),
  generik: (
    <>
      <path d="M20 4C11 4 5 8.5 5 15.5c0 1.6.4 3 1 4.5 1.5.6 2.9 1 4.5 1C17.5 21 20 13 20 4z" />
      <path d="M6 20c3-4.5 6.5-8 10-10.5" />
    </>
  ),
  cek: <path d="M5 12.5 9.5 17 19 7.5" />,
  panah: <path d="M5 12h14M13 6l6 6-6 6" />,
  video: (
    <>
      <rect x="3" y="6" width="13" height="12" rx="2" />
      <path d="m16 10.5 5-3v9l-5-3" />
    </>
  ),
  perisai: (
    <>
      <path d="M12 3l8 3v6c0 4.5-3.2 8-8 9-4.8-1-8-4.5-8-9V6l8-3z" />
      <path d="M9 12l2 2 4-4" />
    </>
  ),
  unduh: <path d="M12 4v11M7 11l5 5 5-5M5 20h14" />,
  struk: (
    <>
      <path d="M6 3h12v18l-3-2-3 2-3-2-3 2V3z" />
      <path d="M9 8h6M9 12h6M9 16h3" />
    </>
  ),
};

export function Ikon({ nama, kelas }: { nama: NamaIkon; kelas?: string }) {
  return (
    <svg
      className={kelas === undefined ? "situs-ikon" : `situs-ikon ${kelas}`}
      viewBox="0 0 24 24"
      width="24"
      height="24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.75}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      {JALUR[nama]}
    </svg>
  );
}

/** Ikon dalam lingkaran mint (pola DG §5.2; ikon `primary-strong` di atas mint 6,56:1, N3). */
export function IkonLingkar({ nama, kelas }: { nama: NamaIkon; kelas?: string }) {
  return (
    <span className={kelas === undefined ? "situs-ikon-lingkar" : `situs-ikon-lingkar ${kelas}`} aria-hidden="true">
      <Ikon nama={nama} />
    </span>
  );
}
