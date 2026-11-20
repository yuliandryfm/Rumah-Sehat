import { Card } from "@rs/ui";
import Link from "next/link";
import type { KontenBeranda } from "../konten/tipe.ts";
import { TeksAtauTidakDiketahui, TidakDiketahui } from "./TidakDiketahui.tsx";

/** Isi Beranda (wireframe s01 bagian 1): hero, ubin layanan, Untuk Perusahaan, artikel terbaru. */
export function Beranda({ konten }: { konten: KontenBeranda }) {
  return (
    <>
      <div className="situs-hero">
        <div>
          <h1>
            <TeksAtauTidakDiketahui teks={konten.judulUtama} ganti="[Judul utama dari CMS]" />
          </h1>
          <p>
            <TeksAtauTidakDiketahui teks={konten.kalimatPembuka} ganti="[Kalimat pembuka tentang Ruang Sehat dari CMS.]" />
          </p>
          <div className="situs-cta">
            <Link className="rs-btn rs-btn--primary" href="/unduh" prefetch={false}>
              Unduh aplikasi
            </Link>
            <Link className="rs-btn rs-btn--secondary" href="/untuk-perusahaan" prefetch={false}>
              Untuk Perusahaan
            </Link>
          </div>
        </div>
        <div className="situs-gambar" role="img" aria-label="Placeholder gambar utama; gambar final dari CMS dengan teks alternatif">
          Gambar utama (placeholder)
        </div>
      </div>
      <section className="situs-sec" aria-labelledby="b-layanan">
        <h2 id="b-layanan">Layanan</h2>
        <div className="situs-kartu">
          {konten.layanan.map((l) => (
            <Card key={l.judul}>
              <h3>{l.judul}</h3>
              <p className="situs-kecil">
                <TeksAtauTidakDiketahui teks={l.uraian} ganti="[Uraian dari CMS]" />
              </p>
            </Card>
          ))}
        </div>
        <p>
          <Link href="/layanan" prefetch={false}>
            Lihat semua layanan
          </Link>
        </p>
      </section>
      <section className="situs-sec situs-sec--alt" aria-labelledby="b-korporat">
        <h2 id="b-korporat">Untuk Perusahaan</h2>
        <p>
          <TeksAtauTidakDiketahui teks={konten.ringkasanKorporat} ganti="[Ringkasan layanan korporat dari CMS.]" />
        </p>
        <p>
          <Link className="rs-btn rs-btn--secondary" href="/untuk-perusahaan" prefetch={false}>
            Pelajari layanan perusahaan
          </Link>
        </p>
      </section>
      <section className="situs-sec" aria-labelledby="b-artikel">
        <h2 id="b-artikel">Artikel terbaru</h2>
        {konten.artikelTerbaru.length === 0 ? (
          <p>
            Belum ada artikel. <TidakDiketahui />
          </p>
        ) : (
          <ul className="situs-kartu situs-daftar">
            {konten.artikelTerbaru.map((a) => (
              <li key={a.slug}>
                <Card>
                  <span className="situs-cap">{a.tanggalTayang ?? "[Tanggal]"}</span>
                  <h3>{a.judul}</h3>
                </Card>
              </li>
            ))}
          </ul>
        )}
      </section>
    </>
  );
}
