"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import type { ItemNav } from "../lib/navigasi.ts";

/** Navigasi samping. Klien hanya untuk menandai halaman aktif (`aria-current`); tautan tetap biasa. */
export function NavSamping({ menu }: { menu: readonly ItemNav[] }) {
  const jalur = usePathname();
  const aktif = (href: string) => (href === "/" ? jalur === "/" : jalur === href || jalur.startsWith(`${href}/`));
  return (
    <nav className="bo-side" aria-label="Menu">
      {menu.map((i) => (
        <Link key={i.href} href={i.href} prefetch={false} aria-current={aktif(i.href) ? "page" : undefined}>
          {i.label}
        </Link>
      ))}
    </nav>
  );
}
