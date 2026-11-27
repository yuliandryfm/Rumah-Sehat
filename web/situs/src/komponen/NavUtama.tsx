"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { NAV_UTAMA } from "../navigasi.ts";

/** Navigasi utama. Klien hanya untuk menandai halaman aktif (`aria-current`); tautannya tetap tautan biasa. */
export function NavUtama({ label }: { label: string }) {
  const jalur = usePathname();
  const aktif = (href: string) => (href === "/" ? jalur === "/" : jalur === href || jalur.startsWith(`${href}/`));
  return (
    <nav className="situs-nav" aria-label={label}>
      {NAV_UTAMA.map((n) => (
        <Link key={n.href} href={n.href} prefetch={false} aria-current={aktif(n.href) ? "page" : undefined}>
          {n.label}
        </Link>
      ))}
    </nav>
  );
}
