"use client";

import { type KeyboardEvent, useRef, useSyncExternalStore } from "react";
import { type KartuLayanan, type KategoriLayanan, kategoriLayanan, TAB_KATEGORI } from "../konten/struktur.ts";

export type Pilihan = "semua" | KategoriLayanan;

/** Tab yang ditampilkan: "Semua" + kategori yang punya kartu; kosong bila kurang dari dua kategori (filter tak berguna). */
export function bangunTab(kartu: readonly Pick<KartuLayanan, "ikon">[], labelSemua: string): { id: Pilihan; label: string }[] {
  const tab: { id: Pilihan; label: string }[] = [{ id: "semua", label: labelSemua }, ...TAB_KATEGORI.filter((t) => kartu.some((k) => kategoriLayanan(k.ikon) === t.id))];
  return tab.length >= 3 ? tab : [];
}

export function cocokKategori(ikon: KartuLayanan["ikon"], pilihan: Pilihan): boolean {
  return pilihan === "semua" || kategoriLayanan(ikon) === pilihan;
}

/**
 * Tab filter kategori layanan (WAI-ARIA tabs, aktivasi otomatis): panah kiri/kanan berputar, Home/End, tabindex
 * berjalan. Dipakai penggeser Beranda dan daftar halaman Layanan.
 */
export function TabKategori({
  tab,
  pilihan,
  onPilih,
  idDasar,
  idPanel,
}: {
  tab: readonly { id: Pilihan; label: string }[];
  pilihan: Pilihan;
  onPilih: (p: Pilihan) => void;
  idDasar: string;
  idPanel: string;
}) {
  const tabRef = useRef(new Map<Pilihan, HTMLButtonElement>());

  function tombol(e: KeyboardEvent<HTMLDivElement>) {
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
    onPilih(tujuan.id);
    tabRef.current.get(tujuan.id)?.focus();
  }

  return (
    <div className="situs-slider__tab" role="tablist" aria-label="Filter kategori layanan" onKeyDown={tombol}>
      {tab.map((t) => (
        <button
          key={t.id}
          ref={(el) => {
            if (el === null) tabRef.current.delete(t.id);
            else tabRef.current.set(t.id, el);
          }}
          type="button"
          role="tab"
          id={idTab(idDasar, t.id)}
          aria-selected={pilihan === t.id}
          aria-controls={idPanel}
          tabIndex={pilihan === t.id ? 0 : -1}
          className="situs-slider__tab-satu"
          onClick={() => {
            onPilih(t.id);
          }}
        >
          {t.label}
        </button>
      ))}
    </div>
  );
}

export const idTab = (dasar: string, p: Pilihan) => `${dasar}-tab-${p}`;

const langgananKosong = () => () => {
  // Nilai tidak berubah setelah hidrasi: tidak ada yang perlu dilanggan.
};
/** `false` saat render server dan hidrasi pertama, `true` sesudahnya: kontrol hanya muncul bila JS berjalan. */
export function useSiap(): boolean {
  return useSyncExternalStore(
    langgananKosong,
    () => true,
    () => false,
  );
}
