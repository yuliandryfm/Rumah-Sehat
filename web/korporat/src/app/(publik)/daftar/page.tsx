import type { Metadata } from "next";
import { FormDaftar } from "../../../komponen/daftar/FormDaftar.tsx";

export const metadata: Metadata = { title: "Daftarkan perusahaan" };

/**
 * Pendaftaran perusahaan mandiri (F1-T103, US-1901, Q30 = C). Halaman publik: tanpa `wajibSesi`, tanpa cookie,
 * tidak menyentuh Valkey atau OpenBao. Formulir memanggil API publik langsung dari browser (KEPUTUSAN-372).
 */
export default function HalamanDaftar() {
  return (
    <div className="kp-publik">
      <a className="kp-skip" href="#isi">
        Lewati ke isi
      </a>
      <header className="kp-top">
        <span className="kp-logo">Ruang Sehat &middot; Daftarkan perusahaan</span>
      </header>
      <main id="isi" tabIndex={-1} className="kp-main">
        <FormDaftar />
      </main>
    </div>
  );
}
