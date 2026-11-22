"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import type { ItemNav } from "../lib/navigasi.ts";

/**
 * Navigasi samping. Klien hanya untuk menandai halaman aktif (`aria-current`); tautan tetap biasa.
 * Layanan yang belum ada tampil sebagai teks nonaktif dengan alasan tertulis "belum tersedia" (K01 layar 9).
 */
export function NavSamping({ menu, belumTersedia }: { menu: readonly ItemNav[]; belumTersedia: readonly string[] }) {
  const jalur = usePathname();
  const aktif = (href: string) => (href === "/" ? jalur === "/" : jalur === href || jalur.startsWith(`${href}/`));
  return (
    <nav className="kp-side" aria-label="Menu">
      {menu.map((i) => (
        <Link key={i.href} href={i.href} prefetch={false} aria-current={aktif(i.href) ? "page" : undefined}>
          {i.label}
        </Link>
      ))}
      {belumTersedia.length === 0 ? null : <span className="kp-sec">Layanan korporat</span>}
      {belumTersedia.map((nama) => (
        <span key={nama} className="kp-nonaktif" aria-disabled="true">
          {nama} (belum tersedia)
        </span>
      ))}
    </nav>
  );
}
