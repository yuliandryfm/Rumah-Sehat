import { ambilUrlKorporat } from "../konten/tautan-korporat.ts";
import { IkonLingkar } from "./Ikon.tsx";

/**
 * Blok "Sudah terdaftar?" di halaman Untuk Perusahaan (CR-032). Tidak dirender bila `SITUS_KORPORAT_URL`
 * kosong atau tidak valid. Tombol "Daftarkan perusahaan" menyusul bersama F1-T103.
 */
export function MasukMemberArea({ url = ambilUrlKorporat() }: { url?: string | null }) {
  if (url === null) return null;
  return (
    <section className="situs-sec situs-sec--rapat" aria-labelledby="sudah-terdaftar">
      <div className="situs-baca">
        <div className="situs-blok-aksi">
          <IkonLingkar nama="korporat" />
          <div className="situs-tumpuk situs-tumpuk--rapat">
            <h2 id="sudah-terdaftar">Sudah terdaftar?</h2>
            <p className="situs-kecil">Member area adalah aplikasi terpisah dari situs ini.</p>
          </div>
          <a className="rs-btn rs-btn--primary" href={url} rel="noopener noreferrer">
            Masuk member area
          </a>
        </div>
      </div>
    </section>
  );
}
