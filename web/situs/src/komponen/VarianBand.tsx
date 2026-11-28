"use client";

import { usePathname } from "next/navigation";
import type { ReactNode } from "react";

interface TeksBand {
  judul: string;
  isi: string;
}

/**
 * Variasi teks pita unduh per halaman, mengikuti desain Stitch. Teks di kode, faktual, bersumber dokumen: tujuh
 * layanan dipakai di aplikasi (BRIEF), keluhan diajukan dan dipantau di aplikasi (PRD US-1103), peserta korporat
 * menautkan akun lewat undangan di aplikasi (PRD AB-85). Klaim "gratis" dari desain tidak dipakai (belum ada
 * keputusan bisnis), KEPUTUSAN-480.
 */
const TEKS_UMUM: TeksBand = {
  judul: "Mulai hidup lebih sehat bersama Ruang Sehat",
  isi: "Tersedia untuk perangkat Android dan iOS. Tujuh layanan kesehatan dalam satu aplikasi.",
};
const TEKS_HALAMAN: Readonly<Record<string, TeksBand>> = {
  "/layanan": {
    judul: "Semua layanan dipesan dan dipakai di aplikasi Ruang Sehat",
    isi: "Unduh aplikasi untuk mulai. Tersedia untuk perangkat Android dan iOS.",
  },
  "/kontak": {
    judul: "Ajukan keluhan lebih cepat lewat aplikasi",
    isi: "Ajukan dan pantau penanganan keluhan langsung dari aplikasi Ruang Sehat di Android dan iOS.",
  },
  "/untuk-perusahaan": {
    judul: "Aplikasi Ruang Sehat untuk peserta",
    isi: "Peserta menerima undangan dari perusahaan lalu menautkan akunnya di aplikasi Ruang Sehat.",
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
