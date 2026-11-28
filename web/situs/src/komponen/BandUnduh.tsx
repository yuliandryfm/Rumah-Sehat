import { SembunyikanDiUnduh } from "./SembunyikanDiUnduh.tsx";
import { TautanToko } from "./TautanToko.tsx";

/**
 * Pita ajakan unduh di akhir isi setiap halaman (CR-042, desain Stitch "App Download CTA Band"). Teks tetap di
 * kode dan faktual (BRIEF: Android dan iOS, tujuh layanan dalam satu aplikasi); tanpa klaim harga ("gratis" tidak
 * dipakai karena belum ada keputusan bisnisnya). Tautan toko memakai logika yang sama dengan header (CR-038):
 * alamat dari env, jatuh ke halaman Unduh aplikasi bila belum diatur. Tidak tampil di halaman Unduh aplikasi.
 */
export function BandUnduh() {
  return (
    <SembunyikanDiUnduh>
      <section className="situs-band" aria-labelledby="band-unduh-judul">
        <div className="situs-wadah">
          <div className="situs-band__kartu">
            <div className="situs-band__teks">
              <h2 id="band-unduh-judul">Mulai hidup lebih sehat bersama Ruang Sehat</h2>
              <p>Tersedia untuk perangkat Android dan iOS. Tujuh layanan kesehatan dalam satu aplikasi.</p>
            </div>
            <TautanToko varian="band" />
          </div>
        </div>
      </section>
    </SembunyikanDiUnduh>
  );
}
