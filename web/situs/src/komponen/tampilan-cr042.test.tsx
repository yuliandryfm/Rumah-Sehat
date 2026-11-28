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
    // Teks mockup apa adanya atas keputusan user (2026-11-28 "A", KEPUTUSAN-494).
    expect(band.textContent).toContain("Tersedia gratis untuk perangkat Android dan iOS. Semua layanan kesehatan terpadu dalam genggaman.");
  });

  test("alamat toko valid dipakai di pita; pita tetap ada di halaman Unduh aplikasi (header tidak lagi memuat tautan toko)", () => {
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
    expect(within(screen.getByRole("region", { name: "Mulai hidup lebih sehat bersama Ruang Sehat" })).getByRole("link", { name: "Unduh di Google Play" })).toBeDefined();
  });

  test("teks pita bervariasi per halaman, persis teks mockup (keputusan user 2026-11-28 \"A\")", () => {
    const hasil: Record<string, string> = {};
    const hasilIsi: Record<string, string> = {};
    for (const j of ["/layanan", "/kontak", "/untuk-perusahaan", "/artikel"]) {
      jalur.nilai = j;
      render(
        <KerangkaSitus>
          <p>isi</p>
        </KerangkaSitus>,
      );
      const band = within(screen.getByRole("main")).getByRole("region");
      hasil[j] = band.querySelector("h2")?.textContent ?? "";
      hasilIsi[j] = band.querySelector("p")?.textContent ?? "";
      cleanup();
    }
    expect(hasil).toEqual({
      "/layanan": "Semua layanan dipesan dan dipakai di aplikasi Ruang Sehat",
      "/kontak": "Ajukan keluhan lebih cepat lewat aplikasi",
      "/untuk-perusahaan": "Aplikasi Ruang Sehat untuk peserta",
      "/artikel": "Mulai hidup lebih sehat bersama Ruang Sehat",
    });
    expect(hasilIsi).toEqual({
      "/layanan": "Unduh aplikasi untuk mulai. Tersedia gratis untuk perangkat Android dan iOS.",
      "/kontak": "Pantau status penanganan keluhan langsung dari aplikasi Ruang Sehat di Android dan iOS.",
      "/untuk-perusahaan": "Karyawan menerima undangan lewat email dan memakai aplikasi Ruang Sehat untuk jadwal serta hasil pemeriksaan.",
      "/artikel": "Tersedia gratis untuk perangkat Android dan iOS. Semua layanan kesehatan terpadu dalam genggaman.",
    });
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
    const kartu = [...container.querySelectorAll(".situs-slider__daftar > li")];
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

describe("tampilan baru (CR-042 lanjutan): aksi header (KEPUTUSAN-491)", () => {
  test("satu tombol Unduh aplikasi ke /unduh di header; pil toko hanya di pita (varian band)", () => {
    vi.stubEnv("SITUS_PLAYSTORE_URL", "");
    vi.stubEnv("SITUS_APPSTORE_URL", "");
    vi.stubEnv("SITUS_KORPORAT_URL", "");
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    const header = screen.getByRole("banner");
    expect(within(header).queryByRole("group", { name: "Unduh aplikasi" })).toBeNull();
    const unduh = within(header).getByRole("link", { name: "Unduh aplikasi" });
    expect(unduh.getAttribute("href")).toBe("/unduh");
    expect(unduh.className).toBe("situs-aksi__unduh");
    expect(unduh.querySelector("svg")?.getAttribute("aria-hidden")).toBe("true");
    expect(within(header).queryByRole("link", { name: "Masuk HR" })).toBeNull();
    const grupBand = within(screen.getByRole("main")).getByRole("group", { name: "Unduh aplikasi" });
    expect(grupBand.className).toBe("situs-toko situs-toko--band");
    expect(css).toMatch(/\.situs-aksi__unduh \{[^}]*background: var\(--c-primary\);[^}]*color: var\(--c-on-primary\);/);
  });

  test("Masuk HR di header (tautan teks sebelum tombol unduh) hanya bila alamat portal valid (CR-032)", () => {
    vi.stubEnv("SITUS_KORPORAT_URL", "https://member.contoh.test/");
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    const header = screen.getByRole("banner");
    const masuk = within(header).getByRole("link", { name: "Masuk HR" });
    expect(masuk.getAttribute("href")).toBe("https://member.contoh.test/");
    expect(masuk.getAttribute("rel")).toBe("noopener noreferrer");
    const aksi = masuk.parentElement;
    expect([...(aksi?.children ?? [])].map((e) => e.textContent)).toEqual(["Masuk HR", "Unduh aplikasi"]);
  });
});

describe("tampilan baru (CR-042): navigasi seluler dan gerak", () => {
  test("di bawah 1024 px navigasi utama tidak tampil (desain: hidden lg:flex, tanpa hamburger) dan header tetap lengket", () => {
    const sempit = css.includes("@media (max-width: 1023px) {\n  .situs-nav-baris { display: none; }");
    expect(sempit).toBe(true);
    const ponsel = /@media \(max-width: 767px\) \{([\s\S]*?)\n\}/.exec(css)?.[1] ?? "";
    expect(ponsel).not.toMatch(/\.situs-top \{ position: static; \}/);
    expect(css).toMatch(/\.situs-top \{ position: sticky;/);
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
