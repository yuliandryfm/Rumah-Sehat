import { ambilUrlKorporat } from "../konten/tautan-korporat.ts";

/**
 * Blok "Sudah terdaftar?" di halaman Untuk Perusahaan (CR-032). Tidak dirender bila `SITUS_KORPORAT_URL`
 * kosong atau tidak valid. Tombol "Daftarkan perusahaan" menyusul bersama F1-T103.
 */
export function MasukMemberArea({ url = ambilUrlKorporat() }: { url?: string | null }) {
  if (url === null) return null;
  return (
    <section className="situs-sec situs-baca" aria-labelledby="sudah-terdaftar">
      <h2 id="sudah-terdaftar">Sudah terdaftar?</h2>
      <p>
        <a className="rs-btn rs-btn--primary" href={url} rel="noopener noreferrer">
          Masuk member area
        </a>
      </p>
      <p className="situs-kecil">Member area adalah aplikasi terpisah dari situs ini.</p>
    </section>
  );
}
