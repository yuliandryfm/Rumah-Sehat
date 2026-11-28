"use client";

import Link from "next/link";
import { type KeyboardEvent, type ReactNode, useCallback, useEffect, useId, useRef, useState, useSyncExternalStore } from "react";
import { type KartuLayanan, type KategoriLayanan, kategoriLayanan, TAB_KATEGORI } from "../konten/struktur.ts";
import { Ikon, IkonLingkar } from "./Ikon.tsx";

/** Kartu yang dikirim ke komponen klien: hanya data yang dapat diserialisasi (tanpa node Markdown). */
export type KartuSlider = Omit<KartuLayanan, "sisa">;
type Pilihan = "semua" | KategoriLayanan;

const langgananKosong = () => () => {
  // Nilai tidak berubah setelah hidrasi: tidak ada yang perlu dilanggan.
};
/** `false` saat render server dan hidrasi pertama, `true` sesudahnya: kontrol hanya muncul bila JS berjalan. */
function useSiap(): boolean {
  return useSyncExternalStore(
    langgananKosong,
    () => true,
    () => false,
  );
}

function kurangiGerak(): boolean {
  return typeof window.matchMedia === "function" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/** Panel kartu: foto lokal bila slot terisi (TD-69), selain itu ilustrasi ikon dekoratif. */
function PanelKartu({ kartu }: { kartu: KartuSlider }) {
  if (kartu.gambar === undefined) return <IkonLingkar nama={kartu.ikon} />;
  return (
    <span className="situs-layanan__foto">
      <img src={kartu.gambar.src} alt={kartu.gambar.alt} loading="lazy" />
    </span>
  );
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
  const tabRef = useRef(new Map<Pilihan, HTMLButtonElement>());

  const tab: { id: Pilihan; label: string }[] = [
    { id: "semua", label: "Semua" },
    ...TAB_KATEGORI.filter((t) => kartu.some((k) => kategoriLayanan(k.ikon) === t.id)),
  ];
  // Filter hanya berguna bila ada dua kategori atau lebih.
  const adaTab = tab.length >= 3;
  const tampil = (k: KartuSlider) => pilihan === "semua" || kategoriLayanan(k.ikon) === pilihan;
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

  function tombolTab(e: KeyboardEvent<HTMLDivElement>) {
    const i = tab.findIndex((t) => t.id === pilihan);
    let baru: number;
    if (e.key === "ArrowRight") baru = (i + 1) % tab.length;
    else if (e.key === "ArrowLeft") baru = (i - 1 + tab.length) % tab.length;
    else if (e.key === "Home") baru = 0;
    else if (e.key === "End") baru = tab.length - 1;
    else return;
    e.preventDefault();
    const tujuan = tab[baru];
    if (tujuan === undefined) return;
    pilih(tujuan.id);
    tabRef.current.get(tujuan.id)?.focus();
  }

  const idTab = (p: Pilihan) => `${id}-tab-${p}`;
  const idPanel = `${id}-panel`;
  const sebagaiTab = siap && adaTab;

  return (
    <div className="situs-slider">
      <div className="situs-slider__kepala">
        {kepala}
        <div className="situs-slider__kontrol" hidden={!siap}>
          {adaTab ? (
            <div className="situs-slider__tab" role="tablist" aria-label="Filter kategori layanan" onKeyDown={tombolTab}>
              {tab.map((t) => (
                <button
                  key={t.id}
                  ref={(el) => {
                    if (el === null) tabRef.current.delete(t.id);
                    else tabRef.current.set(t.id, el);
                  }}
                  type="button"
                  role="tab"
                  id={idTab(t.id)}
                  aria-selected={pilihan === t.id}
                  aria-controls={idPanel}
                  tabIndex={pilihan === t.id ? 0 : -1}
                  className="situs-slider__tab-satu"
                  onClick={() => {
                    pilih(t.id);
                  }}
                >
                  {t.label}
                </button>
              ))}
            </div>
          ) : null}
          <div className="situs-slider__panah">
            <button
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
        aria-labelledby={sebagaiTab ? idTab(pilihan) : undefined}
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
