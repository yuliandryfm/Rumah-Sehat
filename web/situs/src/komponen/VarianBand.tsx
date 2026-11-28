"use client";

import { usePathname } from "next/navigation";
import type { ReactNode } from "react";

interface TeksBand {
  judul: string;
  isi: string;
}

/**
 * Variasi teks pita unduh per halaman, mengikuti desain Stitch. Teks mockup apa adanya atas keputusan user
 * (`[USER]` 2026-11-28 "A (pakai teks mockup apa adanya)", KEPUTUSAN-494): termasuk "gratis" (keputusan bisnis
 * milik user), "Pantau status penanganan keluhan" (PRD US-1103), dan "undangan lewat email" (PRD AB-85).
 */
const TEKS_UMUM: TeksBand = {
  judul: "Mulai hidup lebih sehat bersama Ruang Sehat",
  isi: "Tersedia gratis untuk perangkat Android dan iOS. Semua layanan kesehatan terpadu dalam genggaman.",
};
const TEKS_HALAMAN: Readonly<Record<string, TeksBand>> = {
  "/layanan": {
    judul: "Semua layanan dipesan dan dipakai di aplikasi Ruang Sehat",
    isi: "Unduh aplikasi untuk mulai. Tersedia gratis untuk perangkat Android dan iOS.",
  },
  "/kontak": {
    judul: "Ajukan keluhan lebih cepat lewat aplikasi",
    isi: "Pantau status penanganan keluhan langsung dari aplikasi Ruang Sehat di Android dan iOS.",
  },
  "/untuk-perusahaan": {
    judul: "Aplikasi Ruang Sehat untuk peserta",
    isi: "Karyawan menerima undangan lewat email dan memakai aplikasi Ruang Sehat untuk jadwal serta hasil pemeriksaan.",
  },
};

/** Klien hanya untuk membaca jalur; tautan toko (anak) tetap dirender server. */
export function VarianBand({ children }: { children: ReactNode }) {
  const teks = TEKS_HALAMAN[usePathname()] ?? TEKS_UMUM;
  return (
    <section className="situs-band" aria-labelledby="band-unduh-judul">
      <div className="situs-wadah">
        <div className="situs-band__kartu">
          <div className="situs-band__teks">
            <h2 id="band-unduh-judul">{teks.judul}</h2>
            <p>{teks.isi}</p>
          </div>
          {children}
        </div>
      </div>
    </section>
  );
}
