"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { LABEL_SEGERA_HADIR, type ItemMenu } from "../lib/navigasi.ts";

const cocok = (jalur: string, rute: string): boolean =>
  rute === "/" ? jalur === "/" : jalur === rute || jalur.startsWith(`${rute}/`);

/**
 * Navigasi. Klien hanya untuk menandai halaman aktif (`aria-current`); tautan tetap biasa.
 * Item tanpa halaman tampil sebagai teks abu-abu "segera hadir" (`aria-disabled`), bukan tautan (CR-039).
 */
export function NavSamping({ menu }: { menu: readonly ItemMenu[] }) {
  const jalur = usePathname();
  const aktif = (i: ItemMenu) => [i.href, ...(i.cakup ?? [])].some((r) => r !== undefined && cocok(jalur, r));
  return (
    <nav className="kp-side" aria-label="Menu">
      {menu.map((i) =>
        i.href === undefined ? (
          <span key={i.label} className="kp-nonaktif" aria-disabled="true">
            {i.label} <span className="kp-kecil">({LABEL_SEGERA_HADIR})</span>
          </span>
        ) : (
          <Link key={i.label} href={i.href} prefetch={false} aria-current={aktif(i) ? "page" : undefined}>
            {i.label}
          </Link>
        ),
      )}
    </nav>
  );
}
