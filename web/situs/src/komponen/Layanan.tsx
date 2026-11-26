import Link from "next/link";
import type { KartuLayanan, UraianLayanan } from "../konten/struktur.ts";
import { uraiLayanan } from "../konten/struktur.ts";
import type { Konten } from "../konten/tipe.ts";
import { HalamanKonten, HeroHalaman } from "./HalamanKonten.tsx";
import { Ikon, IkonLingkar } from "./Ikon.tsx";
import { BlokMarkdown } from "./Markdown.tsx";

function kelasKartu(k: KartuLayanan, rinci: boolean): string {
  const dasar = rinci ? "situs-layanan situs-layanan--rinci" : "situs-layanan";
  return k.ikon === "korporat" ? `${dasar} situs-layanan--korporat` : dasar;
}

/**
 * Kartu layanan ringkas di Beranda (CR-033 C-02): ikon, judul, ringkasan, "Selengkapnya". Seluruh kartu dapat
 * diklik lewat satu tautan di judul (perluasan area dengan CSS), sehingga hanya ada satu tautan per kartu dan
 * cincin fokus tampil di kartu (`:focus-within`). Tujuan: jangkar kartu rinci di halaman Layanan.
 */
export function GridLayanan({ kartu }: { kartu: readonly KartuLayanan[] }) {
  return (
    <ul className="situs-grid-layanan situs-daftar">
      {kartu.map((k) => (
        <li key={k.id} className={kelasKartu(k, false)}>
          <IkonLingkar nama={k.ikon} />
          <h3 className="situs-layanan__judul">
            <Link className="situs-tautan-kartu" href={`/layanan#${k.id}`} prefetch={false}>
              {k.judul}
            </Link>
          </h3>
          <p className="situs-layanan__ringkas">{k.ringkasan}</p>
          <span className="situs-layanan__lanjut" aria-hidden="true">
            Selengkapnya <Ikon nama="panah" />
          </span>
        </li>
      ))}
    </ul>
  );
}

/** Kartu rinci halaman Layanan: ringkasan dan sisa isi bagian (Markdown aman). */
function GridLayananRinci({ uraian }: { uraian: UraianLayanan }) {
  return (
    <ul className="situs-grid-layanan situs-grid-layanan--rinci situs-daftar">
      {uraian.kartu.map((k) => (
        <li key={k.id} id={k.id} className={kelasKartu(k, true)}>
          <IkonLingkar nama={k.ikon} />
          <h2 className="situs-layanan__judul">{k.judul}</h2>
          <p className="situs-layanan__ringkas">{k.ringkasan}</p>
          <BlokMarkdown nodes={k.sisa} kelas="situs-md situs-layanan__isi" judulMulai={3} />
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
