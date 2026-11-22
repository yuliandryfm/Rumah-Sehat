import { Parser, type Node } from "commonmark";
import type { ReactNode } from "react";
import { gambarDiizinkan, tautanDiizinkan } from "../konten/aturan-aman.ts";

/**
 * Render Markdown CMS ke elemen React (SIT-01, ADR-027 butir 5, TD-25). Aman karena rancangannya:
 * - Pengurai commonmark.js (CommonMark 0.31.2, tanpa ekstensi, sama dengan pulldown-cmark `Options::empty()`
 *   di backend) hanya menghasilkan pohon; keluaran dibangun sebagai elemen React, tidak pernah string HTML,
 *   sehingga tidak ada penyisipan HTML mentah ke DOM dan semua teks di-escape React.
 * - HTML mentah (blok maupun sebaris) tidak pernah menjadi elemen: ditampilkan sebagai teks biasa.
 * - Tautan hanya berskema https/mailto/relatif dan gambar hanya `/v1/public/content/images/<uuid>`
 *   (`aturan-aman.ts`, sama dengan validasi backend F1-T111); yang melanggar tidak menjadi tautan atau gambar.
 * - Tautan keluar membawa `rel="noopener noreferrer"`; tanpa atribut `style`, `class`, atau `on*` dari konten.
 * - Judul isi bergeser satu tingkat (`#` menjadi h2) karena h1 halaman adalah judul konten.
 */
export function Markdown({ teks }: { teks: string }) {
  const dokumen = new Parser().parse(teks);
  return <div className="situs-md">{anak(dokumen, false)}</div>;
}

function anak(node: Node, ketat: boolean): ReactNode[] {
  const hasil: ReactNode[] = [];
  let n = node.firstChild;
  let i = 0;
  while (n !== null) {
    hasil.push(<NodeMd key={i} node={n} ketat={ketat} />);
    n = n.next;
    i += 1;
  }
  return hasil;
}

/** Teks polos seluruh isi node (untuk teks alternatif gambar). */
function teksPolos(node: Node): string {
  let s = "";
  const jalan = node.walker();
  for (let e = jalan.next(); e !== null; e = jalan.next()) {
    if (e.entering && (e.node.type === "text" || e.node.type === "code")) s += e.node.literal ?? "";
    if (e.entering && (e.node.type === "softbreak" || e.node.type === "linebreak")) s += " ";
  }
  return s;
}

const JUDUL = ["h2", "h3", "h4", "h5", "h6", "h6"] as const;

function NodeMd({ node, ketat }: { node: Node; ketat: boolean }): ReactNode {
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
      return <em>{anak(node, ketat)}</em>;
    case "strong":
      return <strong>{anak(node, ketat)}</strong>;
    case "link": {
      const tujuan = node.destination ?? "";
      if (!tautanDiizinkan(tujuan)) return <>{anak(node, ketat)}</>;
      const keluar = /^(?:https|mailto):/i.test(tujuan);
      return (
        <a href={tujuan} {...(keluar ? { rel: "noopener noreferrer" } : {})}>
          {anak(node, ketat)}
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
      return ketat ? <>{anak(node, ketat)}</> : <p>{anak(node, ketat)}</p>;
    case "heading": {
      const Tag = JUDUL[Math.min(Math.max(node.level, 1), 6) - 1] ?? "h6";
      return <Tag>{anak(node, ketat)}</Tag>;
    }
    case "block_quote":
      return <blockquote>{anak(node, false)}</blockquote>;
    case "list": {
      const tight = node.listTight;
      if (node.listType === "ordered") {
        const mulai = node.listStart;
        return <ol {...(mulai !== 1 ? { start: mulai } : {})}>{anak(node, tight)}</ol>;
      }
      return <ul>{anak(node, tight)}</ul>;
    }
    case "item":
      return <li>{anak(node, ketat)}</li>;
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
