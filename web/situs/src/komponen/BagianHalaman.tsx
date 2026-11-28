import type { Node } from "commonmark";
import { ikonUntukJudul, teksPolos } from "../konten/struktur.ts";
import { IkonLingkar } from "./Ikon.tsx";
import { BlokMarkdown } from "./Markdown.tsx";

/**
 * Tata letak bagian halaman gaya desain Stitch (CR-042 lanjutan, KEPUTUSAN-492) yang digerakkan oleh struktur
 * Markdown CMS, tanpa teks karangan:
 * - setiap judul `tingkat` (`##` di halaman, `###` di halaman detail layanan) menjadi satu bagian bertajuk h2
 *   dengan latar bergantian (app / putih bergaris);
 * - dua sub-judul atau lebih di dalam bagian menjadi kartu (grid auto-fit, sudut 28), bernomor bila judul bagian
 *   memuat "cara"/"langkah"/"alur" (pola "tiga langkah"), akordeon `<details>` bila memuat "tanya"/"FAQ";
 * - `susunan="kartu"` menjadikan seluruh bagian satu grid kartu (pola Kontak: tiga kartu saluran);
 * - node sebelum judul pertama menjadi kolom baca biasa.
 * Semua isi tetap dirender lewat `BlokMarkdown` yang aman (tidak ada HTML mentah, tanpa gaya sebaris).
 */
interface Bagian {
  judul: string;
  pembuka: Node[];
  kartu: { judul: string; isi: Node[] }[];
}

const POLA_LANGKAH = /\b(cara|langkah|alur)\b/i;
const POLA_TANYA = /tanya|faq|pertanyaan/i;

export function pecahBagian(nodes: readonly Node[], tingkat: number): { awal: Node[]; bagian: Bagian[] } {
  const awal: Node[] = [];
  const bagian: Bagian[] = [];
  let sub: { judul: string; isi: Node[] } | null = null;
  for (const n of nodes) {
    if (n.type === "heading" && n.level === tingkat) {
      bagian.push({ judul: teksPolos(n).replace(/\s+/g, " ").trim(), pembuka: [], kartu: [] });
      sub = null;
      continue;
    }
    const kini = bagian.at(-1);
    if (kini === undefined) {
      awal.push(n);
      continue;
    }
    if (n.type === "heading" && n.level === tingkat + 1) {
      sub = { judul: teksPolos(n).replace(/\s+/g, " ").trim(), isi: [] };
      kini.kartu.push(sub);
      continue;
    }
    if (sub === null) kini.pembuka.push(n);
    else sub.isi.push(n);
  }
  return { awal, bagian };
}

function Kartu({ judul, isi, nomor }: { judul: string; isi: readonly Node[]; nomor: number | null }) {
  return (
    <li className="situs-kartu-info">
      {nomor === null ? (
        <IkonLingkar nama={ikonUntukJudul(judul)} />
      ) : (
        <span className="situs-nomor" aria-hidden="true">
          {nomor}
        </span>
      )}
      <h3>{judul}</h3>
      <BlokMarkdown nodes={isi} kelas="situs-md situs-kartu-info__isi" judulMulai={3} />
    </li>
  );
}

function Tanya({ judul, isi }: { judul: string; isi: readonly Node[] }) {
  return (
    <details className="situs-tanya">
      <summary>{judul}</summary>
      <BlokMarkdown nodes={isi} kelas="situs-md situs-tanya__isi" judulMulai={3} />
    </details>
  );
}

export function BagianHalaman({ nodes, tingkat = 2, susunan = "bagian" }: { nodes: readonly Node[]; tingkat?: number; susunan?: "bagian" | "kartu" }) {
  const { awal, bagian } = pecahBagian(nodes, tingkat);
  const nomorHeading = (i: number) => `bag-${String(i + 1)}`;
  return (
    <>
      {awal.length === 0 ? null : (
        <div className="situs-sec situs-sec--rapat">
          <div className="situs-baca">
            <BlokMarkdown nodes={awal} />
          </div>
        </div>
      )}
      {susunan === "kartu" && bagian.length > 0 ? (
        <section className="situs-sec situs-sec--rapat">
          <div className="situs-wadah">
            <ul className="situs-kartu-grid situs-daftar">
              {bagian.map((b) => (
                <li key={b.judul} className="situs-kartu-info">
                  <IkonLingkar nama={ikonUntukJudul(b.judul)} />
                  <h2>{b.judul}</h2>
                  <BlokMarkdown nodes={[...b.pembuka, ...b.kartu.flatMap((k) => k.isi)]} kelas="situs-md situs-kartu-info__isi" judulMulai={3} />
                </li>
              ))}
            </ul>
          </div>
        </section>
      ) : (
        bagian.map((b, i) => {
          const tanya = POLA_TANYA.test(b.judul) && b.kartu.length > 0;
          const bernomor = POLA_LANGKAH.test(b.judul);
          return (
            <section key={`${b.judul}-${String(i)}`} className={i % 2 === 0 ? "situs-sec" : "situs-sec situs-sec--putih"} aria-labelledby={nomorHeading(i)}>
              <div className="situs-wadah situs-tumpuk situs-tumpuk--lebar">
                <div className="situs-kepala-sec">
                  <h2 id={nomorHeading(i)}>{b.judul}</h2>
                  <BlokMarkdown nodes={b.pembuka} kelas={b.kartu.length === 0 ? "situs-md" : "situs-md situs-lead"} />
                </div>
                {b.kartu.length === 0 ? null : tanya ? (
                  <div className="situs-tanya-daftar">
                    {b.kartu.map((k) => (
                      <Tanya key={k.judul} judul={k.judul} isi={k.isi} />
                    ))}
                  </div>
                ) : (
                  <ul className="situs-kartu-grid situs-daftar">
                    {b.kartu.map((k, j) => (
                      <Kartu key={k.judul} judul={k.judul} isi={k.isi} nomor={bernomor ? j + 1 : null} />
                    ))}
                  </ul>
                )}
              </div>
            </section>
          );
        })
      )}
    </>
  );
}
