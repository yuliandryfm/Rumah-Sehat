"use client";

import Link from "next/link";
import { useId, useState } from "react";
import { kategoriLayanan, TAB_KATEGORI } from "../konten/struktur.ts";
import { Ikon } from "./Ikon.tsx";
import { PanelKartu } from "./PanelKartu.tsx";
import type { KartuSlider } from "./SliderLayanan.tsx";
import { bangunTab, cocokKategori, idTab, type Pilihan, TabKategori, useSiap } from "./TabKategori.tsx";

const labelKategori = (k: KartuSlider): string | null => {
  const kat = kategoriLayanan(k.ikon);
  return TAB_KATEGORI.find((t) => t.id === kat)?.label ?? null;
};

/**
 * Daftar layanan halaman Layanan (desain Stitch `layanan_ruang_sehat`, KEPUTUSAN-492): tab filter "Semua layanan /
 * Konsultasi / Klinik & Lab / Home & Belanja", penghitung jumlah dari data nyata, grid dua kolom berkartu setinggi
 * sama (panel gambar, lencana kategori, judul, ringkasan pendek, tautan "Lihat detail layanan"). Isi lengkap ada di
 * halaman detail. Tanpa JS semua kartu tampil dan tab tersembunyi.
 */
export function DaftarLayanan({ kartu }: { kartu: readonly KartuSlider[] }) {
  const id = useId();
  const siap = useSiap();
  const [pilihan, setPilihan] = useState<Pilihan>("semua");
  const tab = bangunTab(kartu, "Semua layanan");
  const sebagaiTab = siap && tab.length > 0;
  const tampil = (k: KartuSlider) => cocokKategori(k.ikon, pilihan);
  const jumlah = kartu.filter(tampil).length;
  const idPanel = `${id}-panel`;
  return (
    <div className="situs-daftar-layanan">
      <div className="situs-daftar-layanan__kepala">
        <div className="situs-slider__kontrol" hidden={!siap}>
          {tab.length === 0 ? null : <TabKategori tab={tab} pilihan={pilihan} onPilih={setPilihan} idDasar={id} idPanel={idPanel} />}
        </div>
        <p className="situs-daftar-layanan__jumlah" role="status">
          {jumlah} layanan tersedia
        </p>
      </div>
      <div
        id={idPanel}
        role={sebagaiTab ? "tabpanel" : undefined}
        aria-labelledby={sebagaiTab ? idTab(id, pilihan) : undefined}
      >
        <ul className="situs-grid-layanan situs-grid-layanan--rinci situs-daftar">
          {kartu.map((k) => {
            const kategori = labelKategori(k);
            return (
              <li key={k.id} id={k.id} className="situs-layanan situs-layanan--rinci" hidden={!tampil(k)}>
                <PanelKartu kartu={k} />
                <div className="situs-layanan__baris">
                  {kategori === null ? <span /> : <span className="situs-chip">{kategori}</span>}
                  <Ikon nama={k.ikon} kelas="situs-layanan__ikon" />
                </div>
                <h2 className="situs-layanan__judul">{k.judul}</h2>
                <p className="situs-layanan__ringkas">{k.ringkasan}</p>
                <div className="situs-layanan__kaki">
                  <Link className="situs-tautan-panah" href={`/layanan/${k.id}`} prefetch={false}>
                    Lihat detail layanan <span className="situs-sr">{k.judul}</span>
                    <Ikon nama="panah" />
                  </Link>
                </div>
              </li>
            );
          })}
        </ul>
      </div>
    </div>
  );
}
