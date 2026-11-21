import { Card } from "@rs/ui";
import Link from "next/link";
import type { Konten } from "../konten/tipe.ts";
import { formatTanggal, GambarKonten, IsiKonten } from "./HalamanKonten.tsx";
import { TidakDiketahui } from "./TidakDiketahui.tsx";

/** Halaman Artikel: pengantar dari halaman CMS `articles` (bila ada) dan daftar artikel tayang, terbaru dulu. */
export function DaftarArtikel({ pengantar, artikel, kursorBerikut }: { pengantar: Konten | null; artikel: Konten[]; kursorBerikut: string | null }) {
  return (
    <section className="situs-sec situs-baca" aria-labelledby="a-judul">
      <h1 id="a-judul">{pengantar?.title ?? "Artikel"}</h1>
      {pengantar === null ? null : <IsiKonten konten={pengantar} />}
      {artikel.length === 0 ? (
        <p>
          Belum ada artikel. <TidakDiketahui />
        </p>
      ) : (
        <ul className="situs-kartu situs-daftar">
          {artikel.map((a) => (
            <li key={a.content_id}>
              <Card>
                <span className="situs-cap">{formatTanggal(a.published_at) ?? "[Tanggal]"}</span>
                <h2>
                  <Link href={`/artikel/${a.content_id}`} prefetch={false}>
                    {a.title}
                  </Link>
                </h2>
              </Card>
            </li>
          ))}
        </ul>
      )}
      {kursorBerikut === null ? null : (
        <p>
          <Link className="rs-btn rs-btn--secondary" href={`/artikel?kursor=${encodeURIComponent(kursorBerikut)}`} prefetch={false}>
            Artikel berikutnya
          </Link>
        </p>
      )}
    </section>
  );
}

/** Satu artikel tayang. */
export function DetailArtikel({ artikel }: { artikel: Konten }) {
  return (
    <article className="situs-sec situs-baca">
      <p>
        <Link href="/artikel" prefetch={false}>
          Semua artikel
        </Link>
      </p>
      <h1>{artikel.title}</h1>
      <p className="situs-cap">
        {formatTanggal(artikel.published_at) ?? "[Tanggal]"}
        {artikel.category === null || artikel.category === undefined ? "" : ` · ${artikel.category}`}
      </p>
      <GambarKonten konten={artikel} />
      <IsiKonten konten={artikel} />
    </article>
  );
}
