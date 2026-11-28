"use client";

import Link from "next/link";
import { type ReactNode, useCallback, useEffect, useId, useRef, useState } from "react";
import type { KartuLayanan } from "../konten/struktur.ts";
import { Ikon } from "./Ikon.tsx";
import { PanelKartu } from "./PanelKartu.tsx";
import { bangunTab, cocokKategori, idTab, type Pilihan, TabKategori, useSiap } from "./TabKategori.tsx";

/** Kartu yang dikirim ke komponen klien: hanya data yang dapat diserialisasi (tanpa node Markdown). */
export type KartuSlider = Omit<KartuLayanan, "sisa">;

function kurangiGerak(): boolean {
  return typeof window.matchMedia === "function" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/**
 * Penggeser layanan Beranda (CR-042 lanjutan, desain Stitch `beranda_ruang_sehat`, KEPUTUSAN-490): jalur gulir
 * horizontal dengan scroll-snap, tab filter kategori, dan tombol sebelumnya/berikutnya. Peningkatan bertahap:
 * tanpa JS semua kartu tetap terbaca dan dapat digulir/digeser (tab dan tombol baru muncul setelah hidrasi).
 * Tanpa gaya sebaris dan tanpa skrip sebaris (CSP bernonce tetap).
 */
export function SliderLayanan({ kartu, kepala }: { kartu: readonly KartuSlider[]; kepala: ReactNode }) {
  const id = useId();
  const siap = useSiap();
  const [pilihan, setPilihan] = useState<Pilihan>("semua");
  const [ujung, setUjung] = useState({ awal: true, akhir: false });
  const jalur = useRef<HTMLDivElement>(null);
  const tombolAwal = useRef<HTMLButtonElement>(null);
  const tombolAkhir = useRef<HTMLButtonElement>(null);

  const tab = bangunTab(kartu, "Semua");
  const adaTab = tab.length > 0;
  const tampil = (k: KartuSlider) => cocokKategori(k.ikon, pilihan);
  const jumlah = kartu.filter(tampil).length;

  const ukur = useCallback(() => {
    const el = jalur.current;
    if (el === null) return;
    const maks = el.scrollWidth - el.clientWidth;
    const baru = { awal: el.scrollLeft <= 1, akhir: el.scrollLeft >= maks - 1 };
    setUjung((lama) => (lama.awal === baru.awal && lama.akhir === baru.akhir ? lama : baru));
  }, []);

  useEffect(() => {
    const el = jalur.current;
    if (el === null || typeof ResizeObserver === "undefined") return;
    const pengamat = new ResizeObserver(ukur);
    pengamat.observe(el);
    if (el.firstElementChild !== null) pengamat.observe(el.firstElementChild);
    return () => {
      pengamat.disconnect();
    };
  }, [ukur]);

  // Tombol panah yang sedang fokus lalu nonaktif (sampai di ujung) tidak boleh menjatuhkan fokus: pindah ke
  // tombol lawannya, atau ke jalur bila keduanya nonaktif.
  useEffect(() => {
    const aktif = document.activeElement;
    if (aktif === tombolAwal.current && ujung.awal) (ujung.akhir ? jalur.current : tombolAkhir.current)?.focus();
    else if (aktif === tombolAkhir.current && ujung.akhir) (ujung.awal ? jalur.current : tombolAwal.current)?.focus();
  }, [ujung]);

  function pilih(p: Pilihan) {
    setPilihan(p);
    jalur.current?.scrollTo({ left: 0, behavior: "auto" });
    // Ukur ulang sesudah DOM memuat ulang kartu yang tampil.
    window.requestAnimationFrame(ukur);
  }

  function geser(arah: 1 | -1) {
    const el = jalur.current;
    if (el === null) return;
    const satu = el.querySelector("li:not([hidden])");
    const daftar = el.firstElementChild;
    const celah = daftar === null ? 0 : Number.parseFloat(getComputedStyle(daftar).columnGap);
    const langkah = (satu?.getBoundingClientRect().width ?? 360) + (Number.isNaN(celah) ? 24 : celah);
    el.scrollBy({ left: arah * langkah, behavior: kurangiGerak() ? "auto" : "smooth" });
  }

  const idPanel = `${id}-panel`;
  const sebagaiTab = siap && adaTab;

  return (
    <div className="situs-slider">
      <div className="situs-slider__kepala">
        {kepala}
        <div className="situs-slider__kontrol" hidden={!siap}>
          {adaTab ? <TabKategori tab={tab} pilihan={pilihan} onPilih={pilih} idDasar={id} idPanel={idPanel} /> : null}
          <div className="situs-slider__panah">
            <button
              ref={tombolAwal}
              type="button"
              className="situs-slider__tombol situs-slider__tombol--balik"
              aria-label="Layanan sebelumnya"
              disabled={ujung.awal}
              onClick={() => {
                geser(-1);
              }}
            >
              <Ikon nama="panah" />
            </button>
            <button
              ref={tombolAkhir}
              type="button"
              className="situs-slider__tombol"
              aria-label="Layanan berikutnya"
              disabled={ujung.akhir}
              onClick={() => {
                geser(1);
              }}
            >
              <Ikon nama="panah" />
            </button>
          </div>
        </div>
      </div>
      <div
        ref={jalur}
        id={idPanel}
        className="situs-slider__jalur"
        tabIndex={0}
        role={sebagaiTab ? "tabpanel" : "region"}
        aria-label={sebagaiTab ? undefined : "Daftar layanan"}
        aria-labelledby={sebagaiTab ? idTab(id, pilihan) : undefined}
        onScroll={ukur}
      >
        <ul className="situs-slider__daftar situs-daftar">
          {kartu.map((k) => (
            <li
              key={k.id}
              className={k.ikon === "korporat" ? "situs-layanan situs-slider__kartu situs-layanan--korporat" : "situs-layanan situs-slider__kartu"}
              hidden={!tampil(k)}
            >
              <PanelKartu kartu={k} />
              <h3 className="situs-layanan__judul">
                <Link className="situs-tautan-kartu" href={`/layanan/${k.id}`} prefetch={false}>
                  {k.judul}
                </Link>
              </h3>
              <p className="situs-layanan__ringkas">{k.ringkasan}</p>
              <span className="situs-layanan__lanjut" aria-hidden="true">
                Selengkapnya <Ikon nama="panah" />
              </span>
            </li>
          ))}
        </ul>
      </div>
      <p className="situs-sr" role="status" hidden={!siap}>
        {adaTab ? `Menampilkan ${String(jumlah)} dari ${String(kartu.length)} layanan` : ""}
      </p>
    </div>
  );
}
