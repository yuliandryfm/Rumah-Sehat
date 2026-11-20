import type { ReactNode } from "react";

/** Bingkai layar status tanpa bilah navigasi (akses ditolak, akun nonaktif, galat): `<main id="isi">`. */
export function LayarStatus({ judul, children, kode }: { judul: string; children?: ReactNode; kode?: string | null }) {
  return (
    <main id="isi" tabIndex={-1} className="bo-status">
      <h1>{judul}</h1>
      {children}
      {kode === undefined || kode === null ? null : <p className="bo-ref">Kode referensi: {kode}</p>}
    </main>
  );
}
