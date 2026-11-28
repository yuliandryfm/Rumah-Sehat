import Link from "next/link";
import type { KartuLayanan, UraianLayanan } from "../konten/struktur.ts";
import { uraiLayanan } from "../konten/struktur.ts";
import type { Konten } from "../konten/tipe.ts";
import { BagianHalaman } from "./BagianHalaman.tsx";
import { HalamanKonten, HeroHalaman } from "./HalamanKonten.tsx";
import { Ikon, IkonLingkar } from "./Ikon.tsx";
import { BlokMarkdown } from "./Markdown.tsx";

function kelasKartu(k: KartuLayanan): string {
  const dasar = "situs-layanan situs-layanan--rinci";
  return k.ikon === "korporat" ? `${dasar} situs-layanan--korporat` : dasar;
}

/** Kartu rinci halaman Layanan: ringkasan dan sisa isi bagian (Markdown aman). */
function GridLayananRinci({ uraian }: { uraian: UraianLayanan }) {
  return (
    <ul className="situs-grid-layanan situs-grid-layanan--rinci situs-daftar">
      {uraian.kartu.map((k) => (
        <li key={k.id} id={k.id} className={kelasKartu(k)}>
          <IkonLingkar nama={k.ikon} />
          <h2 className="situs-layanan__judul">{k.judul}</h2>
          <p className="situs-layanan__ringkas">{k.ringkasan}</p>
          <BlokMarkdown nodes={k.sisa} kelas="situs-md situs-layanan__isi" judulMulai={3} />
          <Link className="situs-tautan-panah" href={`/layanan/${k.id}`} prefetch={false}>
            Lihat detail {k.judul} <Ikon nama="panah" />
          </Link>
        </li>
      ))}
    </ul>
  );
}

/**
 * Halaman Layanan (SP-03). Isi `services` yang cocok konvensi `##` menjadi kartu berikon (urutan dari CMS,
 * PRD US-1801 AC2); bila tidak cocok, halaman isi biasa (Markdown utuh, tidak ada isi yang hilang).
 */
export function HalamanLayanan({ konten }: { konten: Konten }) {
  const uraian = uraiLayanan(konten.body);
  if (uraian === null) return <HalamanKonten konten={konten} />;
  const pengantar = uraian.pengantar;
  const pembuka = pengantar[0]?.type === "paragraph" ? pengantar[0] : null;
  const sisaPengantar = pembuka === null ? pengantar : pengantar.slice(1);
  return (
    <article className="situs-halaman">
      <HeroHalaman judul={konten.title} label="Layanan">
        {pembuka === null ? null : <BlokMarkdown nodes={[pembuka]} kelas="situs-md situs-lead" />}
      </HeroHalaman>
      <div className="situs-sec situs-sec--rapat">
        <div className="situs-wadah situs-tumpuk">
          {sisaPengantar.length === 0 ? null : (
            <div className="situs-baca situs-baca--kiri">
              <BlokMarkdown nodes={sisaPengantar} />
            </div>
          )}
          <GridLayananRinci uraian={uraian} />
        </div>
      </div>
    </article>
  );
}

/**
 * Halaman detail satu layanan (desain Stitch detail_layanan_ruang_sehat, KEPUTUSAN-492): pita judul dari judul dan
 * ringkasan kartu CMS, lalu isi kartu (bagian ### menjadi bagian halaman, #### menjadi kartu/langkah/akordeon
 * lewat BagianHalaman). Tidak ada teks karangan: isi di luar CMS tidak muncul.
 */
export function DetailLayanan({ kartu }: { kartu: KartuLayanan }) {
  return (
    <article className="situs-halaman">
      <HeroHalaman judul={kartu.judul} label="Layanan">
        <p className="situs-lead">{kartu.ringkasan}</p>
      </HeroHalaman>
      <BagianHalaman nodes={kartu.sisa} tingkat={3} />
      <div className="situs-sec situs-sec--rapat">
        <div className="situs-wadah">
          <Link className="situs-tautan-panah situs-tautan-panah--balik" href="/layanan" prefetch={false}>
            <Ikon nama="panah" /> Semua layanan
          </Link>
        </div>
      </div>
    </article>
  );
}
