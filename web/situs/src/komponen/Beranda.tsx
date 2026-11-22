import { Card } from "@rs/ui";
import Link from "next/link";
import type { Konten } from "../konten/tipe.ts";
import { formatTanggal, GambarKonten, IsiKonten } from "./HalamanKonten.tsx";
import { TidakDiketahui } from "./TidakDiketahui.tsx";

export interface DataBeranda {
  /** Halaman `home` yang sedang tayang; `null` bila belum ada (tampil penanda, bukan teks karangan). */
  beranda: Konten | null;
  layanan: Konten | null;
  untukPerusahaan: Konten | null;
  artikelTerbaru: Konten[];
}

/** Isi Beranda (wireframe s01 bagian 1): hero, Layanan, Untuk Perusahaan, artikel terbaru; semua dari CMS. */
export function Beranda({ data }: { data: DataBeranda }) {
  const { beranda, layanan, untukPerusahaan, artikelTerbaru } = data;
  return (
    <>
      <div className="situs-hero">
        <div>
          {beranda === null ? (
            <>
              <h1>
                [Judul utama dari CMS] <TidakDiketahui />
              </h1>
              <p>
                [Kalimat pembuka tentang Ruang Sehat dari CMS.] <TidakDiketahui />
              </p>
            </>
          ) : (
            <>
              <h1>{beranda.title}</h1>
              <IsiKonten konten={beranda} />
            </>
          )}
          <div className="situs-cta">
            <Link className="rs-btn rs-btn--primary" href="/unduh" prefetch={false}>
              Unduh aplikasi
            </Link>
            <Link className="rs-btn rs-btn--secondary" href="/untuk-perusahaan" prefetch={false}>
              Untuk Perusahaan
            </Link>
          </div>
        </div>
        {(beranda?.image ?? null) !== null ? (
          <div className="situs-gambar">
            {beranda === null ? null : <GambarKonten konten={beranda} />}
          </div>
        ) : null}
      </div>
      <section className="situs-sec" aria-labelledby="b-layanan">
        <h2 id="b-layanan">Layanan</h2>
        <p>
          {layanan === null ? (
            <>
              [Ringkasan layanan dari CMS.] <TidakDiketahui />
            </>
          ) : (
            layanan.title
          )}
        </p>
        <p>
          <Link href="/layanan" prefetch={false}>
            Lihat semua layanan
          </Link>
        </p>
      </section>
      <section className="situs-sec situs-sec--alt" aria-labelledby="b-korporat">
        <h2 id="b-korporat">Untuk Perusahaan</h2>
        <p>
          {untukPerusahaan === null ? (
            <>
              [Ringkasan layanan korporat dari CMS.] <TidakDiketahui />
            </>
          ) : (
            untukPerusahaan.title
          )}
        </p>
        <p>
          <Link className="rs-btn rs-btn--secondary" href="/untuk-perusahaan" prefetch={false}>
            Pelajari layanan perusahaan
          </Link>
        </p>
      </section>
      <section className="situs-sec" aria-labelledby="b-artikel">
        <h2 id="b-artikel">Artikel terbaru</h2>
        {artikelTerbaru.length === 0 ? (
          <p>
            Belum ada artikel. <TidakDiketahui />
          </p>
        ) : (
          <ul className="situs-kartu situs-daftar">
            {artikelTerbaru.map((a) => (
              <li key={a.content_id}>
                <Card>
                  <span className="situs-cap">{formatTanggal(a.published_at) ?? "[Tanggal]"}</span>
                  <h3>
                    <Link href={`/artikel/${a.content_id}`} prefetch={false}>
                      {a.title}
                    </Link>
                  </h3>
                </Card>
              </li>
            ))}
          </ul>
        )}
      </section>
    </>
  );
}
