import { Button } from "@rs/ui";
import type { ReactNode } from "react";
import { menuUntuk, type ItemNav } from "../lib/navigasi.ts";
import { NavSamping } from "./NavSamping.tsx";

export interface ProfilAdmin {
  readonly email: string;
  readonly subPeran: readonly string[];
}

export interface KerangkaBackofficeProps {
  /**
   * Profil admin. `null` bila belum ada sumbernya: endpoint profil di API Rust belum ada, jadi sub-peran
   * `[TIDAK DIKETAHUI]` di BFF dan hanya menu untuk semua admin yang tampil.
   */
  readonly profil: ProfilAdmin | null;
  readonly csrfToken: string;
  readonly daftarMenu?: readonly ItemNav[];
  readonly children: ReactNode;
}

/**
 * Tata letak backoffice (wireframe C01 layar 2 dan 3): tautan lewati, bilah atas dengan identitas dan
 * "Keluar", navigasi samping, `<main id="isi">`. Keluar = formulir POST dengan field `csrf_token`.
 */
export function KerangkaBackoffice({ profil, csrfToken, daftarMenu, children }: KerangkaBackofficeProps) {
  const menu = menuUntuk(profil?.subPeran ?? [], daftarMenu);
  return (
    <div className="bo-app">
      <a className="bo-skip" href="#isi">
        Lewati ke isi
      </a>
      <header className="bo-top">
        <span className="bo-logo">Ruang Sehat &middot; Backoffice</span>
        <span className="bo-grow" />
        {profil === null ? null : (
          <span className="bo-kecil">
            {profil.subPeran.join(", ")} &middot; {profil.email}
          </span>
        )}
        <form method="post" action="/auth/logout">
          <input type="hidden" name="csrf_token" value={csrfToken} />
          <Button type="submit" variant="secondary">
            Keluar
          </Button>
        </form>
      </header>
      <NavSamping menu={menu} />
      <main id="isi" tabIndex={-1} className="bo-main">
        {children}
      </main>
    </div>
  );
}
