import { Button } from "@rs/ui";
import type { ReactNode } from "react";
import { ITEM_BELUM_TERSEDIA, ITEM_NAV, type ItemNav } from "../lib/navigasi.ts";
import { NavSamping } from "./NavSamping.tsx";

export interface KerangkaKorporatProps {
  readonly csrfToken: string;
  readonly daftarMenu?: readonly ItemNav[];
  readonly belumTersedia?: readonly string[];
  readonly children: ReactNode;
}

/**
 * Tata letak member area HR (wireframe K01 layar 9): tautan lewati, bilah atas dengan "Keluar", navigasi
 * samping, `<main id="isi">`. Identitas HR dan nama perusahaan belum ditampilkan: endpoint profil di API
 * Rust belum ada `[TIDAK DIKETAHUI]` (F1-T108). Keluar = formulir POST dengan field `csrf_token`.
 */
export function KerangkaKorporat({
  csrfToken,
  daftarMenu = ITEM_NAV,
  belumTersedia = ITEM_BELUM_TERSEDIA,
  children,
}: KerangkaKorporatProps) {
  return (
    <div className="kp-app">
      <a className="kp-skip" href="#isi">
        Lewati ke isi
      </a>
      <header className="kp-top">
        <span className="kp-logo">Ruang Sehat &middot; Member area</span>
        <span className="kp-grow" />
        <form method="post" action="/auth/logout">
          <input type="hidden" name="csrf_token" value={csrfToken} />
          <Button type="submit" variant="secondary">
            Keluar
          </Button>
        </form>
      </header>
      <NavSamping menu={daftarMenu} belumTersedia={belumTersedia} />
      <main id="isi" tabIndex={-1} className="kp-main">
        {children}
      </main>
    </div>
  );
}
