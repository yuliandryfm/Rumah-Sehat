import { Ikon } from "./Ikon.tsx";

/**
 * Banner peringatan keamanan akun di halaman Kontak (desain Stitch `kontak_ruang_sehat`, KEPUTUSAN-492): kartu gelap
 * bersudut 28 dengan ikon perisai. Teks dari desain: peringatan umum agar tidak membagikan password, OTP, atau nomor
 * kartu (selaras ADR-005: tim Ruang Sehat tidak pernah memintanya). Catatan alamat surel contoh pada desain tidak
 * dipakai (situs tidak memuat alamat contoh).
 */
export function BannerKeamanan() {
  return (
    <section className="situs-sec situs-sec--rapat" aria-labelledby="banner-keamanan-judul">
      <div className="situs-wadah">
        <div className="situs-banner-gelap">
          <span className="situs-banner-gelap__ikon" aria-hidden="true">
            <Ikon nama="perisai" />
          </span>
          <div className="situs-banner-gelap__teks">
            <h2 id="banner-keamanan-judul">Jaga keamanan akun Anda</h2>
            <p>Demi keamanan, jangan pernah mengirim password, kode OTP, atau nomor kartu Anda kepada siapa pun, termasuk kepada tim Ruang Sehat.</p>
          </div>
        </div>
      </div>
    </section>
  );
}
