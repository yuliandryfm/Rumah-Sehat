import Link from "next/link";
import { LayarStatus } from "./LayarStatus.tsx";

/**
 * Akses ditolak tanpa membocorkan keberadaan data (wireframe C01 layar 3, KU-03): pesan sama untuk
 * "tidak ada" dan "tidak berhak". Kode referensi = correlation_id (KEPUTUSAN-172).
 */
export function DataTidakDitemukan({ kode }: { kode: string }) {
  return (
    <LayarStatus judul="Data tidak ditemukan" kode={kode}>
      <p>Halaman atau data ini tidak tersedia untuk akun Anda.</p>
      <Link className="rs-btn rs-btn--primary" href="/">
        Kembali ke beranda
      </Link>
    </LayarStatus>
  );
}
