import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { fotoLokalDiizinkan } from "../konten/aturan-aman.ts";
import { uraiLayanan } from "../konten/struktur.ts";
import type { Konten } from "../konten/tipe.ts";
import { BannerKeamanan } from "./BannerKeamanan.tsx";
import { Beranda } from "./Beranda.tsx";
import { HalamanKonten } from "./HalamanKonten.tsx";
import { HalamanLayanan } from "./Layanan.tsx";
import { SliderLayanan } from "./SliderLayanan.tsx";

vi.mock("next/navigation", () => ({ usePathname: () => "/" }));
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

const konten = (body: string, title = "Judul"): Konten => ({
  content_id: "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b",
  content_type: "info_page",
  title,
  body,
  published_at: "2026-11-20T00:00:00Z",
});

const TUJUH = ["Konsultasi dokter", "Konsultasi psikolog", "Pendaftaran klinik", "Pendaftaran lab", "Home visit", "Informasi biaya MCU dan lab", "Belanja produk kesehatan"];
const MD_LAYANAN = ["Pengantar.", ...TUJUH.flatMap((j) => ["", `## ${j}`, "", `Ringkasan ${j}.`, "", "### Rincian rahasia", "", `Isi panjang ${j}.`])].join("\n");

describe("halaman Layanan: tab, penghitung, lencana, kartu ringkas (KEPUTUSAN-492)", () => {
  test("penghitung memakai jumlah data nyata dan berubah menurut filter; tab tersedia", () => {
    render(<HalamanLayanan konten={konten(MD_LAYANAN, "Tujuh layanan")} />);
    expect(screen.getByRole("status").textContent).toBe("7 layanan tersedia");
    const tab = within(screen.getByRole("tablist", { name: "Filter kategori layanan" })).getAllByRole("tab");
    expect(tab.map((t) => t.textContent)).toEqual(["Semua layanan", "Konsultasi", "Klinik & Lab", "Home & Belanja"]);
    fireEvent.click(screen.getByRole("tab", { name: "Klinik & Lab" }));
    expect(screen.getByRole("status").textContent).toBe("3 layanan tersedia");
    fireEvent.click(screen.getByRole("tab", { name: "Konsultasi" }));
    expect(screen.getByRole("status").textContent).toBe("2 layanan tersedia");
  });

  test("penghitung mengikuti jumlah kartu CMS, bukan angka tetap (dua kartu = 2)", () => {
    render(<HalamanLayanan konten={konten("## Konsultasi dokter\n\nA.\n\n## Home visit\n\nB.")} />);
    expect(screen.getByRole("status").textContent).toBe("2 layanan tersedia");
  });

  test("tiap kartu: lencana kategori, ringkasan pendek saja, tautan detail; sisa isi tidak ikut; grid kartu setinggi sama", () => {
    const { container } = render(<HalamanLayanan konten={konten(MD_LAYANAN)} />);
    const kartu = [...container.querySelectorAll(".situs-daftar-layanan .situs-layanan")];
    expect(kartu).toHaveLength(7);
    expect(kartu.map((k) => k.querySelector(".situs-chip")?.textContent)).toEqual(["Konsultasi", "Konsultasi", "Klinik & Lab", "Klinik & Lab", "Home & Belanja", "Klinik & Lab", "Home & Belanja"]);
    expect(container.textContent).not.toContain("Isi panjang");
    expect(container.textContent).not.toContain("Rincian rahasia");
    expect(kartu[0]?.querySelector(".situs-layanan__ringkas")?.textContent).toBe("Ringkasan Konsultasi dokter.");
    expect(within(kartu[0] as HTMLElement).getByRole("link", { name: /Lihat detail layanan/ }).getAttribute("href")).toBe("/layanan/layanan-1");
    expect(container.querySelector(".situs-grid-layanan--rinci")).not.toBeNull();
  });

  test("hero memuat tombol Unduh aplikasi untuk mulai ke /unduh", () => {
    render(<HalamanLayanan konten={konten(MD_LAYANAN)} />);
    expect(screen.getByRole("link", { name: "Unduh aplikasi untuk mulai" }).getAttribute("href")).toBe("/unduh");
  });
});

describe("Kontak: banner keamanan gelap (KEPUTUSAN-492)", () => {
  test("peringatan agar tidak membagikan password, OTP, atau nomor kartu; tanpa catatan alamat surel contoh", () => {
    const { container } = render(<BannerKeamanan />);
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Jaga keamanan akun Anda");
    expect(container.textContent).toMatch(/password, kode OTP, atau nomor kartu/);
    expect(container.textContent).not.toMatch(/contoh|demo/i);
    expect(container.querySelector("svg")?.getAttribute("aria-hidden")).toBe("true");
  });
});

describe("Beranda tanpa pita CMS di bawah hero", () => {
  test("daftar butir di isi `home` tidak dirender (desain tidak punya pita itu)", () => {
    const { container } = render(<Beranda data={{ beranda: konten("Pembuka.\n\n- Klaim satu\n- Klaim dua", "Halo"), layanan: null, untukPerusahaan: null, artikelTerbaru: [] }} />);
    expect(container.textContent).not.toContain("Klaim satu");
  });
});

describe("slot foto kartu: jalur kode nyata dan batasan sumber", () => {
  test.each([
    ["/foto/a.webp", true],
    ["/foto/sub/a.webp", true],
    ["//evil.test/a.webp", false],
    ["https://evil.test/a.webp", false],
    ["javascript:alert(1)", false],
    ["foto/a.webp", false],
    ["/foto/../rahasia", false],
    ["/foto\\a.webp", false],
    ["/foto/a.webp?x=1", false],
    ["/foto/a b.webp", false],
    ["/foto/a\n.webp", false],
    ["", false],
  ])("fotoLokalDiizinkan(%j) = %s", (src, harapan) => {
    expect(fotoLokalDiizinkan(src)).toBe(harapan);
  });

  const FOTO = { konsultasi: { src: "/foto/dokter.webp", alt: "Dokter melayani konsultasi video" } } as const;

  test("uraiLayanan memasang foto dari tabel ke kartu berikon yang sama; Beranda dan Layanan merender <img> beralt", () => {
    const kartu = uraiLayanan(MD_LAYANAN, FOTO)?.kartu ?? [];
    expect(kartu[0]?.gambar).toEqual(FOTO.konsultasi);
    expect(kartu[1]?.gambar).toBeUndefined(); // psikolog: ikon lain
    const { container } = render(<SliderLayanan kartu={kartu} kepala={<h2>x</h2>} />);
    expect(container.querySelectorAll("img")).toHaveLength(1);
    expect(container.querySelector("img")?.getAttribute("alt")).toBe("Dokter melayani konsultasi video");
  });

  test("kartu dengan src berbahaya atau tanpa alt jatuh kembali ke panel ikon (tidak ada <img>)", () => {
    for (const gambar of [
      { src: "//evil.test/a.webp", alt: "Alt" },
      { src: "https://evil.test/a.webp", alt: "Alt" },
      { src: "/foto/a.webp", alt: "  " },
    ]) {
      const kartu = (uraiLayanan(MD_LAYANAN)?.kartu ?? []).map((k, i) => (i === 0 ? { ...k, gambar } : k));
      const { container } = render(<SliderLayanan kartu={kartu} kepala={<h2>x</h2>} />);
      expect(container.querySelector("img"), gambar.src).toBeNull();
      cleanup();
    }
  });
});

describe("penggeser: fokus tidak jatuh saat tombol panah menjadi nonaktif", () => {
  beforeEach(() => {
    Object.defineProperty(HTMLElement.prototype, "scrollBy", { configurable: true, value: vi.fn() });
    Object.defineProperty(HTMLElement.prototype, "scrollTo", { configurable: true, value: vi.fn() });
  });

  test("fokus di 'berikutnya' yang jadi nonaktif pindah ke 'sebelumnya'; di 'sebelumnya' di awal pindah ke 'berikutnya'", () => {
    const kartu = (uraiLayanan(MD_LAYANAN)?.kartu ?? []).map(({ id, judul, ringkasan, ikon }) => ({ id, judul, ringkasan, ikon }));
    const { container } = render(<SliderLayanan kartu={kartu} kepala={<h2>x</h2>} />);
    const jalur = container.querySelector<HTMLElement>(".situs-slider__jalur");
    if (jalur === null) throw new Error("jalur tidak ada");
    const ukur = (scrollLeft: number) => {
      for (const [k, n] of Object.entries({ scrollWidth: 2000, clientWidth: 1000, scrollLeft })) Object.defineProperty(jalur, k, { configurable: true, writable: true, value: n });
      fireEvent.scroll(jalur);
    };
    const sebelumnya = screen.getByRole<HTMLButtonElement>("button", { name: "Layanan sebelumnya" });
    const berikutnya = screen.getByRole<HTMLButtonElement>("button", { name: "Layanan berikutnya" });
    ukur(400);
    berikutnya.focus();
    ukur(1000);
    expect(berikutnya.disabled).toBe(true);
    expect(document.activeElement).toBe(sebelumnya);
    ukur(0);
    expect(sebelumnya.disabled).toBe(true);
    expect(document.activeElement).toBe(berikutnya);
    // Keduanya nonaktif (semua muat): fokus ke jalur gulir.
    ukur(400);
    sebelumnya.focus();
    for (const [k, n] of Object.entries({ scrollWidth: 800, clientWidth: 1000, scrollLeft: 0 })) Object.defineProperty(jalur, k, { configurable: true, writable: true, value: n });
    fireEvent.scroll(jalur);
    expect(document.activeElement).toBe(jalur);
  });
});

describe("BagianHalaman: kartu tanpa kehilangan ###, batas kartu, regex, kunci, tautan berbahaya", () => {
  test('susunan="kartu": teks ### tetap tampil sebagai h3 di dalam kartu', () => {
    const md = "## Pengguna aplikasi\n\nLewat aplikasi.\n\n### Surel\n\nsupport@contoh.test\n\n### Telepon\n\n0800";
    const { container } = render(<HalamanKonten konten={konten(md)} susunan="kartu" />);
    expect([...container.querySelectorAll(".situs-kartu-info h3")].map((h) => h.textContent)).toEqual(["Surel", "Telepon"]);
    expect(container.textContent).toContain("support@contoh.test");
  });

  test("batas kartu = 2: satu ### bukan kartu (h3 di bagian), dua ### menjadi grid kartu", () => {
    const satu = render(<HalamanKonten konten={konten("## Bagian\n\nPengantar.\n\n### Hanya satu\n\nIsi.")} />).container;
    expect(satu.querySelector(".situs-kartu-info")).toBeNull();
    expect(satu.querySelector("h3.situs-sub-judul")?.textContent).toBe("Hanya satu");
    expect(satu.textContent).toContain("Isi.");
    cleanup();
    const dua = render(<HalamanKonten konten={konten("## Bagian\n\n### A\n\nx\n\n### B\n\ny")} />).container;
    expect(dua.querySelectorAll(".situs-kartu-info")).toHaveLength(2);
  });

  test("deteksi cara/tanya memakai batas kata: 'Pencarian' dan 'Caranya' tidak memicu langkah bernomor, 'Cara kerja' memicu", () => {
    const pola = (judul: string) => {
      const { container } = render(<HalamanKonten konten={konten(`## ${judul}\n\n### A\n\nx\n\n### B\n\ny`)} />);
      const n = container.querySelectorAll(".situs-nomor").length;
      cleanup();
      return n;
    };
    expect(pola("Pencarian dokter")).toBe(0);
    expect(pola("Caranya mudah")).toBe(0);
    expect(pola("Cara kerja")).toBe(2);
    const { container } = render(<HalamanKonten konten={konten("## Hal yang sering ditanyakan\n\n### Q?\n\nJawab.")} />);
    expect(container.querySelector("details.situs-tanya")).not.toBeNull();
  });

  test("judul kartu kembar tidak menghasilkan peringatan key React", () => {
    const galat = vi.spyOn(console, "error").mockImplementation(() => undefined);
    render(<HalamanKonten konten={konten("## Sama\n\n### Dup\n\na\n\n### Dup\n\nb\n\n## Sama\n\n### Dup\n\nc\n\n### Dup\n\nd")} />);
    expect(galat).not.toHaveBeenCalled();
  });

  test("tautan javascript: di kartu ####, di akordeon <details>, dan di susunan kartu dibuang (teks tetap)", () => {
    const md = [
      "## Cara memakai",
      "",
      "### Langkah A",
      "",
      "#### Kartu",
      "",
      "[klik](javascript:alert(1)) dan <img src=x onerror=alert(2)>",
      "",
      "### Langkah B",
      "",
      "teks",
      "",
      "## Pertanyaan umum",
      "",
      "### Aman?",
      "",
      "[jawab](javascript:alert(3)) [ok](https://contoh.test/a)",
    ].join("\n");
    const { container } = render(<HalamanKonten konten={konten(md)} />);
    expect(container.querySelector("a[href^='javascript' i]")).toBeNull();
    expect(container.querySelector("img, script")).toBeNull();
    expect(container.querySelector("details a")?.getAttribute("href")).toBe("https://contoh.test/a");
    for (const el of container.querySelectorAll("*")) for (const a of el.getAttributeNames()) expect(a.startsWith("on") || a === "style").toBe(false);
    expect(container.textContent).toContain("klik");
    expect(container.textContent).toContain("jawab");
  });
});
