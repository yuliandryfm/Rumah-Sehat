"use client";

/** Galat pada tata letak akar (mengganti seluruh dokumen). Sengaja minimal: tanpa pesan galat. */
export default function GalatGlobal({ reset }: { error: Error & { digest?: string }; reset: () => void }) {
  return (
    <html lang="id">
      <body>
        <main>
          <h1>Terjadi kesalahan</h1>
          <p>Halaman belum dapat ditampilkan. Silakan coba lagi beberapa saat lagi.</p>
          <button type="button" onClick={reset}>
            Coba lagi
          </button>
        </main>
      </body>
    </html>
  );
}
