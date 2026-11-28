import Link from "next/link";
import type { KartuLayanan } from "../konten/struktur.ts";
import { tanpaSisa, uraiLayanan } from "../konten/struktur.ts";
import type { Konten } from "../konten/tipe.ts";
import { BagianHalaman } from "./BagianHalaman.tsx";
import { DaftarLayanan } from "./DaftarLayanan.tsx";
import { HalamanKonten, HeroHalaman } from "./HalamanKonten.tsx";
import { Ikon } from "./Ikon.tsx";
import { BlokMarkdown } from "./Markdown.tsx";

/**
 * Halaman Layanan (SP-03). Isi `services` yang cocok konvensi `##` menjadi daftar kartu berfilter (urutan dari CMS,
 * PRD US-1801 AC2; `DaftarLayanan`); bila tidak cocok, halaman isi biasa (Markdown utuh, tidak ada isi yang hilang).
 * Isi lengkap tiap layanan ada di halaman detail (`DetailLayanan`).
 */
export function HalamanLayanan({ konten }: { konten: Konten }) {
  const uraian = uraiLayanan(konten.body);
  if (uraian === null) return <HalamanKonten konten={konten} />;
  const pengantar = uraian.pengantar;
  const pembuka = pengantar[0]?.type === "paragraph" ? pengantar[0] : null;
  const sisaPengantar = pembuka === null ? pengantar : pengantar.slice(1);
  return (
    <article className="situs-halaman">
      <HeroHalaman
        judul={konten.title}
        label="Layanan"
        aksi={
          <Link className="situs-btn situs-btn--utama" href="/unduh" prefetch={false}>
            Unduh aplikasi untuk mulai <Ikon nama="panah" />
          </Link>
        }
      >
        {pembuka === null ? null : <BlokMarkdown nodes={[pembuka]} kelas="situs-md situs-lead" />}
      </HeroHalaman>
      <div className="situs-sec situs-sec--rapat">
        <div className="situs-wadah situs-tumpuk">
          {sisaPengantar.length === 0 ? null : (
            <div className="situs-baca situs-baca--kiri">
              <BlokMarkdown nodes={sisaPengantar} />
            </div>
          )}
          <DaftarLayanan kartu={uraian.kartu.map(tanpaSisa)} />
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
