"use client";

import { Button } from "@rs/ui";
import { LayarStatus } from "../komponen/LayarStatus.tsx";
import { kodeReferensi } from "../lib/korelasi.ts";

/** Galat server: pesan umum + kode referensi (KEPUTUSAN-172). Tidak menampilkan pesan atau jejak galat. */
export default function Galat({ error, reset }: { error: Error & { digest?: string }; reset: () => void }) {
  return (
    <LayarStatus judul="Terjadi kesalahan" kode={kodeReferensi(error)}>
      <p>Halaman belum dapat ditampilkan. Sebutkan kode referensi ini bila menghubungi layanan pelanggan.</p>
      <Button onClick={reset}>Coba lagi</Button>
    </LayarStatus>
  );
}
