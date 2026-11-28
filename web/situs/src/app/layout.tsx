import "@rs/ui/tokens.css";
import "@rs/ui/ui.css";
import "./situs.css";
import type { Metadata } from "next";
import { connection } from "next/server";
import type { ReactNode } from "react";
import { KerangkaSitus } from "../komponen/KerangkaSitus.tsx";
import { fontAngka, fontIsi, fontJudul } from "./font.ts";

export const metadata: Metadata = {
  title: { default: "Ruang Sehat", template: "%s - Ruang Sehat" },
};

/**
 * Tata letak akar. `connection()` memaksa render dinamis per permintaan: nonce CSP dari `proxy.ts`
 * hanya dapat dipasang saat render di server (halaman statis tidak punya header permintaan).
 */
export default async function RootLayout({ children }: { children: ReactNode }) {
  await connection();
  return (
    <html lang="id" className={`${fontJudul.variable} ${fontIsi.variable} ${fontAngka.variable}`}>
      <body>
        <KerangkaSitus>{children}</KerangkaSitus>
      </body>
    </html>
  );
}
