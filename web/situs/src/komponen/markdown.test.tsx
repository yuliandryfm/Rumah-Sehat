import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { gambarDiizinkan, tautanDiizinkan } from "../konten/aturan-aman.ts";
import { HalamanKonten } from "./HalamanKonten.tsx";
import { Markdown } from "./Markdown.tsx";

afterEach(cleanup);

const IMG = "/v1/public/content/images/0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";

// Payload dari validasi backend (apps/rs-backend/src/modules/content_support/markdown.rs, F1-T111) ditambah
// payload XSS klasik. Backend menolak semuanya saat simpan; situs tetap tidak boleh membuat elemen berbahaya.
const PAYLOAD_HTML = [
  "<script>alert(1)</script>",
  "teks <script>alert(1)</script> teks",
  '<iframe src="https://example.com"></iframe>',
  "<img src=x onerror=alert(1)>",
  '<a href="javascript:alert(1)">x</a>',
  '<div onclick="alert(1)">x</div>',
  "<svg><script>alert(1)</script></svg>",
  "<!-- komentar -->",
  "<style>body{}</style>",
  "<details open ontoggle=alert(1)>",
  "teks <b onmouseover=alert(1)>b</b>",
  "<math><mtext></mtext></math>",
  "<object data=x></object><embed src=x><form action=x><input></form>",
];
const PAYLOAD_TAUTAN = [
  "[x](javascript:alert(1))",
  "[x](JaVaScRiPt:alert(1))",
  "[x](jav&#x61;script:alert(1))",
  "[x](jav&#97;script:alert(1))",
  "[x](<javascript:alert(1)>)",
  "[x](<java\tscript:alert(1)>)",
  "[x](data:text/html;base64,PHNjcmlwdD4=)",
  "[x](vbscript:msgbox(1))",
  "[x](file:///etc/passwd)",
  "[x](http://example.com)",
  "[x](//example.com/jalur)",
  "[x](\\\\example.com)",
  "[x](https:example.com)",
  "<javascript:alert(1)>",
  "<http://example.com>",
  "[x][r]\n\n[r]: javascript:alert(1)",
  "[x](mailto:)",
];
const PAYLOAD_GAMBAR = [
  "![a](https://img.example.test/x.png)",
  "![a](/v1/public/content/images/bukan-uuid)",
  "![a](/v1/public/content/images/0193A1B2-C3D4-7E5F-8A6B-7C8D9E0F1A2B)",
  `![a](${IMG}/../../x)`,
  "![a](javascript:alert(1))",
  "![a](data:image/png;base64,AAAA)",
  "![a](/v1/backoffice/x.png)",
  `![](${IMG})`,
  `![  ](${IMG})`,
];

const TAG_BOLEH = new Set(["DIV", "P", "H2", "H3", "H4", "H5", "H6", "UL", "OL", "LI", "EM", "STRONG", "CODE", "PRE", "A", "IMG", "BR", "HR", "BLOCKQUOTE"]);
const ATRIBUT_BOLEH = new Set(["href", "rel", "src", "alt", "loading", "decoding", "start", "class"]);

/** Invarian keamanan keluaran: hanya tag dan atribut di daftar izin, setiap `a[href]` dan `img[src]` lolos aturan. */
function periksaAman(wadah: HTMLElement) {
  for (const el of wadah.querySelectorAll("*")) {
    expect(TAG_BOLEH.has(el.tagName), `tag ${el.tagName}: ${wadah.innerHTML}`).toBe(true);
    for (const a of el.getAttributeNames()) expect(ATRIBUT_BOLEH.has(a), `atribut ${a}: ${wadah.innerHTML}`).toBe(true);
  }
  for (const a of wadah.querySelectorAll("a")) expect(tautanDiizinkan(a.getAttribute("href") ?? ""), a.outerHTML).toBe(true);
  for (const i of wadah.querySelectorAll("img")) {
    expect(gambarDiizinkan(i.getAttribute("src") ?? ""), i.outerHTML).toBe(true);
    expect((i.getAttribute("alt") ?? "").trim().length).toBeGreaterThan(0);
  }
}

describe("render Markdown aman (SIT-01, TD-25)", () => {
  test("Markdown biasa dirender: judul bergeser satu tingkat, daftar, tebal, tautan, gambar publik", () => {
    const md = `# Siapa kami\n\n## Bagian\n\nTeks **tebal**, *miring*, \`kode\`.\n\n- [Layanan](/layanan)\n- [Mitra](https://example.com/mitra)\n- [Surel](mailto:halo@example.com)\n\n1. satu\n2. dua\n\n![Foto klinik](${IMG})\n\n> kutipan\n`;
    const { container } = render(<Markdown teks={md} />);
    expect(screen.getByRole("heading", { level: 2, name: "Siapa kami" })).toBeDefined();
    expect(screen.getByRole("heading", { level: 3, name: "Bagian" })).toBeDefined();
    expect(container.querySelector("strong")?.textContent).toBe("tebal");
    expect(container.querySelectorAll("ul li")).toHaveLength(3);
    expect(container.querySelectorAll("ol li")).toHaveLength(2);
    expect(screen.getByRole("link", { name: "Layanan" }).getAttribute("href")).toBe("/layanan");
    expect(screen.getByRole("link", { name: "Layanan" }).getAttribute("rel")).toBeNull();
    expect(screen.getByRole("link", { name: "Mitra" }).getAttribute("rel")).toBe("noopener noreferrer");
    expect(screen.getByRole("img", { name: "Foto klinik" }).getAttribute("src")).toBe(IMG);
    periksaAman(container);
  });

  test("tidak ada h1 dari isi (h1 halaman adalah judul konten)", () => {
    const { container } = render(<Markdown teks={"# Satu\n\n###### Enam"} />);
    expect(container.querySelector("h1")).toBeNull();
    expect(container.querySelector("h6")?.textContent).toBe("Enam");
  });

  test.each(PAYLOAD_HTML)("HTML mentah tidak pernah menjadi elemen: %s", (payload) => {
    const { container } = render(<Markdown teks={payload} />);
    periksaAman(container);
    expect(container.querySelector("script, iframe, style, svg, math, object, embed, form, input, details")).toBeNull();
  });

  test.each(PAYLOAD_TAUTAN)("tautan di luar daftar izin tidak menjadi <a> berbahaya: %j", (payload) => {
    const { container } = render(<Markdown teks={payload} />);
    periksaAman(container);
    expect(container.innerHTML).not.toMatch(/href="\s*(?:javascript|data|vbscript|file|http):/i);
  });

  test.each(PAYLOAD_GAMBAR)("gambar di luar endpoint publik atau tanpa alt tidak dirender: %j", (payload) => {
    const { container } = render(<Markdown teks={payload} />);
    periksaAman(container);
    expect(container.querySelector("img")).toBeNull();
  });

  test("payload di dalam blok kode dan teks tampil sebagai teks, bukan elemen", () => {
    const { container } = render(<Markdown teks={"```\n<script>alert(1)</script>\n```\n\n`<img src=x onerror=alert(1)>`"} />);
    expect(container.querySelector("script, img")).toBeNull();
    expect(container.textContent).toContain("<script>alert(1)</script>");
    periksaAman(container);
  });

  test("tidak ada atribut event, style, atau kelas yang berasal dari konten", () => {
    const { container } = render(<Markdown teks={'<p class="x" style="a" onclick="b">t</p>\n\n[a](/x "judul") **b**'} />);
    periksaAman(container);
    expect(container.querySelector("a")?.getAttribute("title")).toBeNull();
    expect(container.querySelector("[style], [onclick]")).toBeNull();
  });

  test("judul dan gambar konten (DTO) juga di-escape dan disaring", () => {
    const { container } = render(
      <HalamanKonten
        konten={{
          content_id: "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b",
          content_type: "info_page",
          title: "<script>alert(1)</script>",
          body: "x",
          published_at: "2026-11-20T00:00:00Z",
          image: { url: "https://jahat.example/x.png", alt_text: "a" },
        }}
      />,
    );
    expect(container.querySelector("script, img")).toBeNull();
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("<script>alert(1)</script>");
  });

  test("gambar DTO dari endpoint publik dengan alt dirender; tanpa alt tidak", () => {
    const dasar = { content_id: "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b", content_type: "info_page", title: "T", body: "x", published_at: "2026-11-20T00:00:00Z" } as const;
    const { container, rerender } = render(<HalamanKonten konten={{ ...dasar, image: { url: IMG, alt_text: "Gedung" } }} />);
    expect(screen.getByRole("img", { name: "Gedung" }).getAttribute("src")).toBe(IMG);
    rerender(<HalamanKonten konten={{ ...dasar, image: { url: IMG, alt_text: " " } }} />);
    expect(container.querySelector("img")).toBeNull();
  });
});

describe("aturan-aman setara backend (markdown.rs)", () => {
  test("tautanDiizinkan", () => {
    for (const ok of ["", "https://example.com", "relatif/jalur?x=a:b", "?q=1", "#atas", "/layanan", "mailto:a@b.id", "MAILTO:a@b.id"]) expect(tautanDiizinkan(ok), ok).toBe(true);
    for (const buruk of ["javascript:", "http://x.id", "//x.id", "a b", "a\tb", "a\\b", "https:x", "https://", "mailto:", "ftp://x", "data:,x"]) expect(tautanDiizinkan(buruk), buruk).toBe(false);
  });

  test("gambarDiizinkan: tepat jalur publik dan UUID huruf kecil", () => {
    expect(gambarDiizinkan(IMG)).toBe(true);
    for (const buruk of [`${IMG}/x`, `${IMG}?a`, IMG.toUpperCase(), "/v1/public/content/images/", "https://x.id/a.png", `x${IMG}`]) expect(gambarDiizinkan(buruk), buruk).toBe(false);
  });
});
