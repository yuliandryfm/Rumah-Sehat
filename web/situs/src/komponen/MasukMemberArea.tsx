import { ambilUrlKorporat } from "../konten/tautan-korporat.ts";
import { IkonLingkar } from "./Ikon.tsx";

/** Alamat formulir pendaftaran di web korporat (F1-T103): jalur `/daftar` pada origin `SITUS_KORPORAT_URL`. */
export function urlDaftar(urlKorporat: string): string {
  return new URL("/daftar", urlKorporat).href;
}

/**
 * Blok "Belum terdaftar?" (tombol "Daftarkan perusahaan", F1-T103, KEPUTUSAN-352/373) dan "Sudah terdaftar?"
 * (CR-032) di halaman Untuk Perusahaan. Tidak dirender bila `SITUS_KORPORAT_URL` kosong atau tidak valid. Tautan
 * keluar adalah navigasi biasa ke aplikasi lain; tidak ada formulir ke domain korporat (CSP situs tidak berubah).
 * Satu tombol utama per layar (DG §4): pendaftaran utama, masuk sekunder.
 */
export function MasukMemberArea({ url = ambilUrlKorporat() }: { url?: string | null }) {
  if (url === null) return null;
  return (
    <>
      <section className="situs-sec situs-sec--rapat" aria-labelledby="belum-terdaftar">
        <div className="situs-baca">
          <div className="situs-blok-aksi">
            <IkonLingkar nama="korporat" />
            <div className="situs-tumpuk situs-tumpuk--rapat">
              <h2 id="belum-terdaftar">Belum terdaftar?</h2>
              <p className="situs-kecil">Pendaftaran diperiksa Verifikator Ruang Sehat sebelum perusahaan dapat memakai member area.</p>
            </div>
            <a className="rs-btn rs-btn--primary" href={urlDaftar(url)} rel="noopener noreferrer">
              Daftarkan perusahaan
            </a>
          </div>
        </div>
      </section>
      <section className="situs-sec situs-sec--rapat" aria-labelledby="sudah-terdaftar">
        <div className="situs-baca">
          <div className="situs-blok-aksi">
            <IkonLingkar nama="korporat" />
            <div className="situs-tumpuk situs-tumpuk--rapat">
              <h2 id="sudah-terdaftar">Sudah terdaftar?</h2>
              <p className="situs-kecil">Member area adalah aplikasi terpisah dari situs ini.</p>
            </div>
            <a className="rs-btn rs-btn--secondary" href={url} rel="noopener noreferrer">
              Masuk member area
            </a>
          </div>
        </div>
      </section>
    </>
  );
}
