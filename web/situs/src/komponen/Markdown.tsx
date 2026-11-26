import { Parser, type Node } from "commonmark";
import type { ReactNode } from "react";
import { gambarDiizinkan, tautanDiizinkan } from "../konten/aturan-aman.ts";
import { teksPolos } from "../konten/struktur.ts";

/**
 * Render Markdown CMS ke elemen React (SIT-01, ADR-027 butir 5, TD-25). Aman karena rancangannya:
 * - Pengurai commonmark.js (CommonMark 0.31.2, tanpa ekstensi, sama dengan pulldown-cmark `Options::empty()`
 *   di backend) hanya menghasilkan pohon; keluaran dibangun sebagai elemen React, tidak pernah string HTML,
 *   sehingga tidak ada penyisipan HTML mentah ke DOM dan semua teks di-escape React.
 * - HTML mentah (blok maupun sebaris) tidak pernah menjadi elemen: ditampilkan sebagai teks biasa.
 * - Tautan hanya berskema https/mailto/relatif dan gambar hanya `/v1/public/content/images/<uuid>`
 *   (`aturan-aman.ts`, sama dengan validasi backend F1-T111); yang melanggar tidak menjadi tautan atau gambar.
 * - Tautan keluar membawa `rel="noopener noreferrer"`; tanpa atribut `style`, `class`, atau `on*` dari konten.
 * - Judul isi dinormalkan: tingkat judul terdangkal di isi menjadi h2 (h1 halaman adalah judul konten), dan
 *   tingkat lain bergeser sama, sehingga urutan judul tidak melompat (`##` saja → h2, `#` + `##` → h2 + h3;
 *   DG §7.2 judul bertingkat; CR-033 KEPUTUSAN-357).
 */
export function Markdown({ teks }: { teks: string }) {
  const dokumen = new Parser().parse(teks);
  return <div className="situs-md">{anak(dokumen, false, 2 - judulTerdangkal([dokumen]))}</div>;
}

/** Tingkat judul terdangkal (1–6) di seluruh node; 1 bila tidak ada judul (geser bawaan satu tingkat). */
function judulTerdangkal(nodes: readonly Node[]): number {
  let min = 7;
  for (const akar of nodes) {
    const jalan = akar.walker();
    for (let e = jalan.next(); e !== null; e = jalan.next()) {
      if (e.entering && e.node.type === "heading") min = Math.min(min, Math.max(e.node.level, 1));
    }
  }
  return min === 7 ? 1 : min;
}

/**
 * Render sekumpulan node tingkat atas hasil `konten/struktur.ts` (kartu layanan, kalimat pembuka hero) dengan
 * renderer aman yang sama. `kelas` hanya kelas tetap dari kode, bukan dari konten. `judulMulai`: tingkat elemen
 * untuk judul terdangkal di `nodes` (2 di bawah h1 halaman, 3 di dalam kartu berjudul h2).
 */
export function BlokMarkdown({ nodes, kelas = "situs-md", judulMulai = 2 }: { nodes: readonly Node[]; kelas?: string; judulMulai?: 2 | 3 }) {
  if (nodes.length === 0) return null;
  const geser = judulMulai - judulTerdangkal(nodes);
  return (
    <div className={kelas}>
      {nodes.map((n, i) => (
        <NodeMd key={i} node={n} ketat={false} geser={geser} />
      ))}
    </div>
  );
}

function anak(node: Node, ketat: boolean, geser: number): ReactNode[] {
  const hasil: ReactNode[] = [];
  let n = node.firstChild;
  let i = 0;
  while (n !== null) {
    hasil.push(<NodeMd key={i} node={n} ketat={ketat} geser={geser} />);
    n = n.next;
    i += 1;
  }
  return hasil;
}

const JUDUL = ["h2", "h3", "h4", "h5", "h6"] as const;

function NodeMd({ node, ketat, geser }: { node: Node; ketat: boolean; geser: number }): ReactNode {
  switch (node.type) {
    case "text":
      return node.literal;
    case "softbreak":
      return "\n";
    case "linebreak":
      return <br />;
    case "code":
      return <code>{node.literal}</code>;
    case "emph":
      return <em>{anak(node, ketat, geser)}</em>;
    case "strong":
      return <strong>{anak(node, ketat, geser)}</strong>;
    case "link": {
      const tujuan = node.destination ?? "";
      if (!tautanDiizinkan(tujuan)) return <>{anak(node, ketat, geser)}</>;
      const keluar = /^(?:https|mailto):/i.test(tujuan);
      return (
        <a href={tujuan} {...(keluar ? { rel: "noopener noreferrer" } : {})}>
          {anak(node, ketat, geser)}
        </a>
      );
    }
    case "image": {
      const tujuan = node.destination ?? "";
      const alt = teksPolos(node).trim();
      if (!gambarDiizinkan(tujuan) || alt === "") return null;
      return <img src={tujuan} alt={alt} loading="lazy" decoding="async" />;
    }
    case "paragraph":
      return ketat ? <>{anak(node, ketat, geser)}</> : <p>{anak(node, ketat, geser)}</p>;
    case "heading": {
      // Tidak pernah h1 (minimal h2) dan maksimal h6.
      const Tag = JUDUL[Math.min(Math.max(node.level + geser, 2), 6) - 2] ?? "h6";
      return <Tag>{anak(node, ketat, geser)}</Tag>;
    }
    case "block_quote":
      return <blockquote>{anak(node, false, geser)}</blockquote>;
    case "list": {
      const tight = node.listTight;
      if (node.listType === "ordered") {
        const mulai = node.listStart;
        return <ol {...(mulai !== 1 ? { start: mulai } : {})}>{anak(node, tight, geser)}</ol>;
      }
      return <ul>{anak(node, tight, geser)}</ul>;
    }
    case "item":
      return <li>{anak(node, ketat, geser)}</li>;
    case "code_block":
      return (
        <pre>
          <code>{node.literal}</code>
        </pre>
      );
    case "thematic_break":
      return <hr />;
    case "html_block":
    case "html_inline":
      // Backend menolak HTML mentah saat simpan; bila tetap sampai sini, tampil sebagai teks, bukan elemen.
      return node.literal;
    default:
      return null;
  }
}
