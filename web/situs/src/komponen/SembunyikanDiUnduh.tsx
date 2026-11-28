"use client";

import { usePathname } from "next/navigation";
import type { ReactNode } from "react";

/** Menyembunyikan anaknya di halaman Unduh aplikasi (yang sudah berisi ajakan yang sama). Klien hanya untuk membaca jalur. */
export function SembunyikanDiUnduh({ children }: { children: ReactNode }) {
  return usePathname() === "/unduh" ? null : <>{children}</>;
}
