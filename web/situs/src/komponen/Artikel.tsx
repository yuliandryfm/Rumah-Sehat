import Link from "next/link";
import type { Konten } from "../konten/tipe.ts";
import { adaGambar, formatTanggal, GambarKonten, HeroHalaman, IsiKonten } from "./HalamanKonten.tsx";
import { Ikon } from "./Ikon.tsx";
import { TidakDiketahui } from "./TidakDiketahui.tsx";

/**
 * Kartu artikel (CR-033 C-01): chip kategori (bila ada), tanggal, judul. Seluruh kartu dapat diklik lewat satu
 * tautan di judul. Gambar dan ringkasan di kartu adalah C-06 (di luar CR-033).
 */
export function KartuArtikel({ artikel, tingkat }: { artikel: Konten; tingkat: 2 | 3 }) {
  const Judul = tingkat === 2 ? "h2" : "h3";
  const kategori = artikel.category ?? null;
  return (
    <li className="situs-artikel">
      <p className="situs-artikel__meta">
        {kategori === null || kategori.trim() === "" ? null : <span className="situs-chip">{kategori}</span>}
        <span className="situs-cap">{formatTanggal(artikel.published_at) ?? "[Tanggal]"}</span>
      </p>
      <Judul className="situs-artikel__judul">
        <Link className="situs-tautan-kartu" href={`/artikel/${artikel.content_id}`} prefetch={false}>
          {artikel.title}
        </Link>
      </Judul>
      <span className="situs-layanan__lanjut" aria-hidden="true">
        Baca artikel <Ikon nama="panah" />
      </span>
    </li>
  );
}

/** Halaman Artikel: pengantar dari halaman CMS `articles` (bila ada) dan daftar artikel tayang, terbaru dulu. */
export function DaftarArtikel({ pengantar, artikel, kursorBerikut }: { pengantar: Konten | null; artikel: Konten[]; kursorBerikut: string | null }) {
  return (
    <section className="situs-halaman" aria-labelledby="a-judul">
      <div className="situs-halhero">
        <div className="situs-wadah situs-halhero__isi">
          <p className="situs-eyebrow">Edukasi kesehatan</p>
          <h1 id="a-judul">{pengantar?.title ?? "Artikel"}</h1>
          {pengantar === null ? null : (
            <div className="situs-lead">
              <IsiKonten konten={pengantar} />
            </div>
          )}
        </div>
      </div>
      <div className="situs-sec situs-sec--rapat">
        <div className="situs-wadah situs-tumpuk situs-tumpuk--lebar">
          {artikel.length === 0 ? (
            <p>
              Belum ada artikel. <TidakDiketahui />
            </p>
          ) : (
            <ul className="situs-grid-artikel situs-daftar">
              {artikel.map((a) => (
                <KartuArtikel key={a.content_id} artikel={a} tingkat={2} />
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
        </div>
      </div>
    </section>
  );
}

/** Satu artikel tayang. */
export function DetailArtikel({ artikel }: { artikel: Konten }) {
  const kategori = artikel.category ?? null;
  return (
    <article className="situs-halaman">
      <HeroHalaman judul={artikel.title}>
        <p className="situs-artikel__meta">
          {kategori === null || kategori.trim() === "" ? null : <span className="situs-chip">{kategori}</span>}
          <span className="situs-cap">{formatTanggal(artikel.published_at) ?? "[Tanggal]"}</span>
        </p>
      </HeroHalaman>
      <div className="situs-sec situs-sec--rapat">
        <div className="situs-baca situs-tumpuk">
          <p>
            <Link className="situs-tautan-panah situs-tautan-panah--balik" href="/artikel" prefetch={false}>
              <Ikon nama="panah" /> Semua artikel
            </Link>
          </p>
          {adaGambar(artikel) ? (
            <div className="situs-bingkai">
              <GambarKonten konten={artikel} />
            </div>
          ) : null}
          <IsiKonten konten={artikel} />
        </div>
      </div>
    </article>
  );
}
