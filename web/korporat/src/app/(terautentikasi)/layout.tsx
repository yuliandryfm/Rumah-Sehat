import "@rs/bff-auth/session-dialog.css";
import { SessionTimeoutWarning } from "@rs/bff-auth/client";
import type { ReactNode } from "react";
import { KerangkaKorporat } from "../../komponen/KerangkaKorporat.tsx";
import { wajibSesi } from "../../lib/server/sesi.ts";

/** Semua halaman di grup ini butuh sesi BFF; tanpa sesi → login. Dialog batas sesi (KEPUTUSAN-170) dipasang di sini. */
export default async function LayoutTerautentikasi({ children }: { children: ReactNode }) {
  const { csrfToken } = await wajibSesi();
  return (
    <>
      <KerangkaKorporat csrfToken={csrfToken}>
        {children}
      </KerangkaKorporat>
      <SessionTimeoutWarning />
    </>
  );
}
