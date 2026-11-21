import "@rs/ui/tokens.css";
import "@rs/ui/ui.css";
import "./backoffice.css";
import type { Metadata } from "next";
import { connection } from "next/server";
import type { ReactNode } from "react";

export const metadata: Metadata = {
  title: { default: "Backoffice - Ruang Sehat", template: "%s - Backoffice Ruang Sehat" },
  robots: { index: false, follow: false },
};

/**
 * Tata letak akar. `connection()` memaksa render dinamis per permintaan: nonce CSP dari `proxy.ts` hanya
 * dapat dipasang saat render di server, dan halaman backoffice tidak boleh di-cache lintas pengguna.
 */
export default async function RootLayout({ children }: { children: ReactNode }) {
  await connection();
  return (
    <html lang="id">
      <body>{children}</body>
    </html>
  );
}
