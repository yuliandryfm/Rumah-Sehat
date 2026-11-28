import Link from "next/link";
import { pisahPembuka, tanpaSisa, teksPolos, uraiLayanan } from "../konten/struktur.ts";
import type { Konten } from "../konten/tipe.ts";
import { KartuArtikel } from "./Artikel.tsx";
import { adaGambar, GambarKonten } from "./HalamanKonten.tsx";
import { Ikon, IkonLingkar, type NamaIkon } from "./Ikon.tsx";
import { BlokMarkdown } from "./Markdown.tsx";
import { OrnamenGelap, OrnamenHero } from "./Ornamen.tsx";
import { SliderLayanan } from "./SliderLayanan.tsx";
import { TidakDiketahui } from "./TidakDiketahui.tsx";

export interface DataBeranda {
  /** Halaman `home` yang sedang tayang; `null` bila belum ada (tampil penanda, bukan teks karangan). */
  beranda: Konten | null;
  layanan: Konten | null;
  untukPerusahaan: Konten | null;
  artikelTerbaru: Konten[];
}

/**
 * Teks tetap di kode, faktual dan bersumber (CR-033 C-03; anti-halu: tanpa angka pengguna, rating, testimoni,
 * nama rekanan). Klaim publik tetap menunggu persetujuan teks final (brief situs §12 butir 4).
 */
const CHIP_NILAI: readonly { teks: string; sumber: string }[] = [
  { teks: "Tujuh layanan terpadu", sumber: "BRIEF" },
  { teks: "Panggilan video tidak direkam", sumber: "BRD BR-016" },
  { teks: "Android & iOS", sumber: "BRIEF" },
];
/**
 * Kartu mengambang hero: teks mockup apa adanya atas keputusan user (`[USER]` 2026-11-28 "A (pakai teks mockup
 * apa adanya)", KEPUTUSAN-494); klaim bisnisnya milik user.
 */
const KARTU_MENGAMBANG: readonly {
  label: string;
  isi: string;
  ikon: NamaIkon;
  kelas: string;
}[] = [
  { label: "Chat & Video Privat", isi: "Resep digital resmi", ikon: "video", kelas: "situs-mengambang--a" },
  { label: "Biaya MCU terlihat awal", isi: "100% Transparan", ikon: "struk", kelas: "situs-mengambang--b" },
];
const UBIN_PONSEL: readonly NamaIkon[] = ["konsultasi", "psikolog", "klinik", "lab", "home-visit", "belanja"];

/** Ilustrasi UI bergaya (bukan tangkapan layar, bukan data nyata): bingkai ponsel dengan bentuk abstrak. */
function MockupPonsel() {
  return (
    <div className="situs-ponsel" aria-hidden="true">
      <div className="situs-ponsel__layar">
        <span className="situs-ponsel__poni" />
        <span className="situs-ponsel__garis situs-ponsel__garis--judul" />
        <span className="situs-ponsel__garis situs-ponsel__garis--pendek" />
        <span className="situs-ponsel__cari" />
        <span className="situs-ponsel__ubin">
          {UBIN_PONSEL.map((n) => (
            <span key={n} className="situs-ponsel__satu">
              <IkonLingkar nama={n} kelas="situs-ikon-lingkar--kecil" />
              <span className="situs-ponsel__garis situs-ponsel__garis--label" />
            </span>
          ))}
        </span>
        <span className="situs-ponsel__kartu">
          <span className="situs-ponsel__avatar" />
          <span className="situs-ponsel__baris">
            <span className="situs-ponsel__garis situs-ponsel__garis--pendek" />
            <span className="situs-ponsel__garis situs-ponsel__garis--label" />
          </span>
        </span>
      </div>
    </div>
  );
}

/** Visual hero (C-03): gambar CMS dalam bingkai bila ada; selain itu mockup ponsel di atas blob daun. */
function VisualHero({ beranda }: { beranda: Konten | null }) {
  return (
    <div className="situs-visual">
      <OrnamenHero />
      {beranda !== null && adaGambar(beranda) ? (
        <div className="situs-bingkai situs-bingkai--hero">
          <GambarKonten konten={beranda} />
        </div>
      ) : (
        <MockupPonsel />
      )}
      {KARTU_MENGAMBANG.map((k) => (
        <p key={k.label} className={`situs-mengambang ${k.kelas}`}>
          <IkonLingkar nama={k.ikon} kelas="situs-ikon-lingkar--kecil" />
          <span className="situs-mengambang__teks">
            <span className="situs-mengambang__label">{k.label}</span>
            <span className="situs-mengambang__isi">{k.isi}</span>
          </span>
        </p>
      ))}
    </div>
  );
}

/** Isi Beranda (wireframe s01 bagian 1, konsep A brief §7): hero, Layanan, Untuk Perusahaan, artikel terbaru. */
export function Beranda({ data }: { data: DataBeranda }) {
  const { beranda, layanan, untukPerusahaan, artikelTerbaru } = data;
  const isiBeranda = beranda === null ? null : pisahPembuka(beranda.body);
  const pembukaBeranda = isiBeranda?.pembuka ?? null;
  const uraianLayanan = uraiLayanan(layanan?.body);
  const pembukaKorporat = (untukPerusahaan === null ? null : pisahPembuka(untukPerusahaan.body))?.pembuka ?? null;
  const sorotanKorporat = (uraiLayanan(untukPerusahaan?.body)?.kartu ?? []).slice(0, 4);
  return (
    <>
      <div className="situs-hero">
        <div className="situs-wadah situs-hero__grid">
          <div className="situs-hero__teks">
            <p className="situs-eyebrow">Aplikasi kesehatan Ruang Sehat</p>
            {beranda === null ? (
              <>
                <h1>
                  [Judul utama dari CMS] <TidakDiketahui />
                </h1>
                <p className="situs-lead">
                  [Kalimat pembuka tentang Ruang Sehat dari CMS.] <TidakDiketahui />
                </p>
              </>
            ) : (
              <>
                <h1>{beranda.title}</h1>
                {pembukaBeranda === null ? null : <BlokMarkdown nodes={[pembukaBeranda]} kelas="situs-md situs-lead" />}
              </>
            )}
            <div className="situs-cta">
              <Link className="situs-btn situs-btn--utama" href="/unduh" prefetch={false}>
                Unduh aplikasi
              </Link>
              <Link className="situs-btn situs-btn--sekunder" href="/untuk-perusahaan" prefetch={false}>
                Untuk Perusahaan
              </Link>
            </div>
            <ul className="situs-chip-nilai situs-daftar" aria-label="Sorotan Ruang Sehat">
              {CHIP_NILAI.map((c) => (
                <li key={c.teks}>
                  <Ikon nama="cek" />
                  {c.teks}
                </li>
              ))}
            </ul>
          </div>
          <VisualHero beranda={beranda} />
        </div>
      </div>
      <section className="situs-sec" aria-labelledby="b-layanan">
        <div className="situs-wadah situs-tumpuk situs-tumpuk--lebar">
          {uraianLayanan === null ? (
            <div className="situs-kepala-sec">
              <p className="situs-eyebrow">Untuk pasien</p>
              {layanan === null ? (
                <>
                  <h2 id="b-layanan">Layanan</h2>
                  <p className="situs-lead">
                    [Ringkasan layanan dari CMS.] <TidakDiketahui />
                  </p>
                </>
              ) : (
                <h2 id="b-layanan">{layanan.title}</h2>
              )}
            </div>
          ) : (
            <SliderLayanan
              kartu={uraianLayanan.kartu.map(tanpaSisa)}
              kepala={
                <div className="situs-kepala-sec">
                  <p className="situs-eyebrow">Untuk pasien</p>
                  <h2 id="b-layanan">{layanan?.title}</h2>
                </div>
              }
            />
          )}
          <p>
            <Link className="situs-tautan-panah" href="/layanan" prefetch={false}>
              Lihat semua layanan <Ikon nama="panah" />
            </Link>
          </p>
        </div>
      </section>
      <section className="situs-sec situs-sec--gelap" aria-labelledby="b-korporat">
        <div className="situs-wadah situs-korporat">
          <OrnamenGelap />
          <div className="situs-tumpuk">
            <p className="situs-eyebrow">Untuk HR dan perusahaan</p>
            <h2 id="b-korporat">Untuk Perusahaan</h2>
            <p className="situs-lead">
              {untukPerusahaan === null ? (
                <>
                  [Ringkasan layanan korporat dari CMS.] <TidakDiketahui />
                </>
              ) : (
                untukPerusahaan.title
              )}
            </p>
            {pembukaKorporat === null ? null : <p>{teksPolos(pembukaKorporat)}</p>}
            <p className="situs-cta">
              <Link className="situs-btn situs-btn--terang" href="/untuk-perusahaan" prefetch={false}>
                Pelajari layanan perusahaan
              </Link>
            </p>
          </div>
          {sorotanKorporat.length === 0 ? (
            <div className="situs-korporat__hias" aria-hidden="true">
              <IkonLingkar nama="korporat" kelas="situs-ikon-lingkar--besar" />
            </div>
          ) : (
            <div className="situs-kaca">
              <p className="situs-kaca__judul" id="b-korporat-sorotan">
                Di halaman Untuk Perusahaan
              </p>
              <ul className="situs-daftar situs-sorotan" aria-labelledby="b-korporat-sorotan">
                {sorotanKorporat.map((k) => (
                  <li key={k.id}>
                    <Ikon nama="cek" />
                    <span>{k.judul}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </section>
      <section className="situs-sec" aria-labelledby="b-artikel">
        <div className="situs-wadah situs-tumpuk situs-tumpuk--lebar">
          <div className="situs-kepala-baris">
            <div className="situs-kepala-sec">
              <p className="situs-eyebrow">Edukasi kesehatan</p>
              <h2 id="b-artikel">Artikel terbaru</h2>
            </div>
            <Link className="situs-tautan-panah" href="/artikel" prefetch={false}>
              Lihat semua artikel <Ikon nama="panah" />
            </Link>
          </div>
          {artikelTerbaru.length === 0 ? (
            <p>
              Belum ada artikel. <TidakDiketahui />
            </p>
          ) : (
            <ul className="situs-grid-artikel situs-daftar">
              {artikelTerbaru.map((a) => (
                <KartuArtikel key={a.content_id} artikel={a} tingkat={3} />
              ))}
            </ul>
          )}
        </div>
      </section>
    </>
  );
}
