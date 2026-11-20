"use client";

import { kodeReferensi } from "../lib/korelasi.ts";

/** Galat pada tata letak akar (mengganti seluruh dokumen). Minimal: tanpa pesan galat, dengan kode referensi. */
export default function GalatGlobal({ error, reset }: { error: Error & { digest?: string }; reset: () => void }) {
  const kode = kodeReferensi(error);
  return (
    <html lang="id">
      <body>
        <main>
          <h1>Terjadi kesalahan</h1>
          <p>Halaman belum dapat ditampilkan. Silakan coba lagi beberapa saat lagi.</p>
          {kode === null ? null : <p>Kode referensi: {kode}</p>}
          <button type="button" onClick={reset}>
            Coba lagi
          </button>
        </main>
      </body>
    </html>
  );
}
