import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import type { Konten } from "../konten/tipe.ts";
import { DaftarArtikel } from "./Artikel.tsx";
import { Beranda, type DataBeranda } from "./Beranda.tsx";
import { KerangkaSitus } from "./KerangkaSitus.tsx";
import { JALUR_LOGO, Logo } from "./Logo.tsx";

const jalur = vi.hoisted(() => ({ nilai: "/" }));
vi.mock("next/navigation", () => ({ usePathname: () => jalur.nilai }));

const sini = dirname(fileURLToPath(import.meta.url));
const akarRepo = join(sini, "../../../..");

afterEach(() => {
  cleanup();
  jalur.nilai = "/";
  vi.unstubAllEnvs();
});

describe("tampilan baru (CR-042): logo", () => {
  test("jalur logo sama persis dengan berkas SVG dari paket desain (docs/design/aset)", () => {
    const svg = readFileSync(join(akarRepo, "docs/design/aset/logo-ruang-sehat.svg"), "utf8");
    const d = [...svg.matchAll(/\sd="([^"]+)"/g)].map((m) => m[1]);
    expect(d).toEqual([JALUR_LOGO.rumah, JALUR_LOGO.daun, JALUR_LOGO.urat]);
  });

  test("logo dekoratif (aria-hidden, tidak dapat difokus); nama merek ada sebagai teks di header dan footer", () => {
    const { container } = render(<Logo />);
    const svg = container.querySelector("svg");
    expect(svg?.getAttribute("aria-hidden")).toBe("true");
    expect(svg?.getAttribute("focusable")).toBe("false");
    cleanup();
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    expect(within(screen.getByRole("banner")).getByRole("link", { name: "Ruang Sehat" }).getAttribute("href")).toBe("/");
    expect(within(screen.getByRole("contentinfo")).getByText("Ruang Sehat")).toBeDefined();
  });
});

describe("tampilan baru (CR-042): pita ajakan unduh", () => {
  test("tampil di akhir isi (di dalam main) dengan dua tautan toko yang jatuh ke /unduh bila env kosong", () => {
    vi.stubEnv("SITUS_PLAYSTORE_URL", "");
    vi.stubEnv("SITUS_APPSTORE_URL", "");
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    const band = within(screen.getByRole("main")).getByRole("region", { name: "Mulai hidup lebih sehat bersama Ruang Sehat" });
    const tautan = within(band).getAllByRole("link");
    expect(tautan.map((a) => a.textContent)).toEqual(["Google Play", "App Store"]);
    expect(tautan.map((a) => a.getAttribute("href"))).toEqual(["/unduh", "/unduh"]);
    // Tanpa klaim harga yang belum diputuskan dan tanpa angka karangan.
    expect(band.textContent).not.toMatch(/gratis|\d/i);
  });

  test("alamat toko valid dipakai juga di pita; tidak tampil di halaman Unduh aplikasi", () => {
    vi.stubEnv("SITUS_PLAYSTORE_URL", "https://play.google.com/store/apps/details?id=id.contoh.uji");
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    const band = screen.getByRole("region", { name: "Mulai hidup lebih sehat bersama Ruang Sehat" });
    expect(within(band).getByRole("link", { name: "Unduh di Google Play" }).getAttribute("rel")).toBe("noopener noreferrer");
    cleanup();
    jalur.nilai = "/unduh";
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    expect(screen.queryByRole("region", { name: "Mulai hidup lebih sehat bersama Ruang Sehat" })).toBeNull();
  });
});

describe("tampilan baru (CR-042): tanpa sumber daya eksternal", () => {
  test("CSS dan kode situs tidak merujuk CDN, Google Fonts, foto googleusercontent, atau ikon Material Symbols", () => {
    const berkas = [
      join(sini, "../app/situs.css"),
      join(sini, "../app/font.ts"),
      join(sini, "Logo.tsx"),
      join(sini, "KerangkaSitus.tsx"),
      join(sini, "BandUnduh.tsx"),
      join(sini, "Beranda.tsx"),
    ];
    for (const b of berkas) {
      const isi = readFileSync(b, "utf8").replace(/\/\*[\s\S]*?\*\//g, "");
      expect(isi, b).not.toMatch(/googleapis|gstatic|googleusercontent|cdn\.tailwindcss|material-symbols|https?:\/\/(?!www\.w3\.org)/i);
    }
  });

  test("tiga keluarga font desain disajikan lokal (next/font/local) beserta lisensi OFL-nya", () => {
    const font = readFileSync(join(sini, "../app/font.ts"), "utf8");
    expect(font.match(/src: "\.\/fonts\/[A-Za-z0-9-]+\.woff2"/g)).toHaveLength(3);
    for (const f of ["OFL-InstrumentSans.txt", "OFL-Alata.txt", "OFL-SplineSansMono.txt"]) {
      expect(readFileSync(join(sini, "../app/fonts", f), "utf8")).toContain("SIL Open Font License, Version 1.1");
    }
  });
});

describe("tampilan baru (CR-042): isi layanan tetap tujuh layanan rilis 1", () => {
  test("kode situs tidak memuat layanan di luar rilis 1 yang ada di mockup (vaksinasi/infus untuk pasien)", () => {
    const isi = readFileSync(join(sini, "Beranda.tsx"), "utf8") + readFileSync(join(sini, "KerangkaSitus.tsx"), "utf8") + readFileSync(join(sini, "BandUnduh.tsx"), "utf8");
    expect(isi).not.toMatch(/Suntik|Infus|Vaksinasi/);
  });
});

const ID = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
const konten = (isi: Partial<Konten>): Konten => ({
  content_id: ID,
  content_type: "info_page",
  title: "Judul",
  body: null,
  published_at: "2026-11-20T00:00:00Z",
  ...isi,
});
const css = readFileSync(join(sini, "../app/situs.css"), "utf8");
const KOSONG: DataBeranda = { beranda: null, layanan: null, untukPerusahaan: null, artikelTerbaru: [] };

describe("tampilan baru (CR-042): panel ilustrasi kartu layanan dekoratif", () => {
  test("panel ikon aria-hidden, tanpa <img> (foto belum ada, TD-69); nama kartu hanya dari judul", () => {
    const md = "Pengantar.\n\n## Konsultasi dokter\n\nChat dan video.\n\n## Home visit\n\nTenaga medis datang.";
    const { container } = render(<Beranda data={{ ...KOSONG, layanan: konten({ body: md }) }} />);
    const kartu = [...container.querySelectorAll(".situs-grid-layanan > li")];
    expect(kartu).toHaveLength(2);
    for (const k of kartu) {
      const panel = k.querySelector(".situs-ikon-lingkar");
      expect(panel?.getAttribute("aria-hidden")).toBe("true");
      expect(panel?.querySelector("svg")?.getAttribute("focusable")).toBe("false");
      expect(k.querySelector("img")).toBeNull();
      expect(k.querySelectorAll("a")).toHaveLength(1);
    }
    expect(screen.getByRole("link", { name: "Konsultasi dokter" })).toBeDefined();
  });
});

describe("tampilan baru (CR-042): tata letak majalah artikel (KEPUTUSAN-479)", () => {
  const artikel = (n: number) => Array.from({ length: n }, (_, i) => konten({ content_id: `0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a${String(10 + i)}`, content_type: "article", title: `Artikel ${String(i + 1)}` }));
  const unggulan = (n: number) => {
    const { container } = render(<DaftarArtikel pengantar={null} artikel={artikel(n)} kursorBerikut={null} />);
    const hasil = container.querySelectorAll(".situs-grid-artikel > li:first-child:nth-last-child(3)").length;
    cleanup();
    return hasil;
  };

  test("tepat tiga artikel: kartu pertama menjadi unggulan; selain tiga: grid dua kolom biasa", () => {
    expect(unggulan(3)).toBe(1);
    for (const n of [1, 2, 4, 5]) expect(unggulan(n), String(n)).toBe(0);
  });

  test("CSS: unggulan hanya untuk tepat tiga artikel (selector first-child + nth-last-child(3)), dua kolom, satu kolom di bawah 1024 px", () => {
    expect(css).toMatch(/\.situs-grid-artikel > li:first-child:nth-last-child\(3\) \{ grid-row: span 2; \}/);
    expect(css).toMatch(/\.situs-grid-artikel \{[^}]*grid-template-columns: repeat\(2, minmax\(0, 1fr\)\);/);
    const tablet = /@media \(max-width: 1023px\) \{([\s\S]*?)\n\}/.exec(css)?.[1] ?? "";
    expect(tablet).toMatch(/\.situs-grid-artikel \{ grid-template-columns: minmax\(0, 1fr\); \}/);
    expect(tablet).toMatch(/\.situs-grid-artikel > li:first-child:nth-last-child\(3\) \{ grid-row: auto; \}/);
  });
});

describe("tampilan baru (CR-042): pil toko di header", () => {
  test("dua pil di banner (Google Play, App Store) memakai kelas pil terisi; pil band memakai varian band", () => {
    vi.stubEnv("SITUS_PLAYSTORE_URL", "");
    vi.stubEnv("SITUS_APPSTORE_URL", "");
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    const grupHeader = within(screen.getByRole("banner")).getByRole("group", { name: "Unduh aplikasi" });
    expect(grupHeader.className).toBe("situs-toko");
    const pil = within(grupHeader).getAllByRole("link");
    expect(pil.map((a) => a.textContent)).toEqual(["Google Play", "App Store"]);
    for (const a of pil) expect(a.className).toBe("situs-toko__tautan");
    const grupBand = within(screen.getByRole("main")).getByRole("group", { name: "Unduh aplikasi" });
    expect(grupBand.className).toBe("situs-toko situs-toko--band");
    expect(css).toMatch(/\.situs-toko__tautan \{[^}]*background: var\(--c-primary\);[^}]*color: var\(--c-on-primary\);/);
  });
});

describe("tampilan baru (CR-042): navigasi seluler dan gerak", () => {
  test("di bawah 768 px navigasi membungkus (tanpa terpotong atau gulir tersembunyi) dan header tidak lengket", () => {
    const ponsel = /@media \(max-width: 767px\) \{([\s\S]*?)\n\}/.exec(css)?.[1] ?? "";
    expect(ponsel).toMatch(/\.situs-nav-baris \{[^}]*overflow: visible;[^}]*mask-image: none;/);
    expect(ponsel).toMatch(/\.situs-nav \{ flex-wrap: wrap; \}/);
    expect(ponsel).toMatch(/\.situs-top \{ position: static; \}/);
  });

  test("setiap hover yang menggeser (transform translate) dimatikan di blok prefers-reduced-motion", () => {
    const blok = /@media \(prefers-reduced-motion: reduce\) \{([\s\S]*)\n\}/.exec(css)?.[1] ?? "";
    const dasarHover = [...css.replace(/@media \(prefers-reduced-motion[\s\S]*$/, "").matchAll(/(\.[a-z0-9_-]+(?:--[a-z0-9-]+)?):hover(?:[^{]*)\{[^}]*transform: translate/g)].map((m) => m[1] ?? "");
    expect(dasarHover.length).toBeGreaterThan(0);
    for (const k of new Set(dasarHover)) {
      const induk = k.replace(/--[a-z0-9-]+$/, "");
      expect(blok.includes(`${k}:hover`) || blok.includes(`${induk}:hover`), k).toBe(true);
    }
    expect(blok).toMatch(/\.situs-btn:hover[^{]*\{ transform: none; \}/);
    // Pita unduh dan pil toko tidak punya hover bergeser.
    expect(css).not.toMatch(/\.situs-band[^{]*:hover[^{]*\{[^}]*transform/);
    expect(css).not.toMatch(/\.situs-toko[^{]*:hover[^{]*\{[^}]*transform/);
  });
});
