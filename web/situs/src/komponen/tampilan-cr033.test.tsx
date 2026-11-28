import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import type { Konten } from "../konten/tipe.ts";
import { Beranda, type DataBeranda } from "./Beranda.tsx";
import { HalamanKonten } from "./HalamanKonten.tsx";
import { HalamanLayanan } from "./Layanan.tsx";

afterEach(cleanup);

const ID = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
const IMG = `/v1/public/content/images/${ID}`;
const LAYANAN_MD = [
  "Semua layanan ada di aplikasi.",
  "",
  "## Konsultasi dokter",
  "",
  "Chat dan video dengan [dokter](/kontak) rekanan.",
  "",
  "- Resep digital",
  "",
  "## Home visit",
  "",
  "Tenaga medis datang ke rumah.",
  "",
  "## Layanan korporat",
  "",
  "Untuk perusahaan.",
  "",
  "## Kolaborasi",
  "",
  "Tanpa kata kunci ikon.",
].join("\n");

const dasar = (isi: Partial<Konten>): Konten => ({
  content_id: ID,
  content_type: "info_page",
  title: "Judul",
  body: null,
  published_at: "2026-11-20T00:00:00Z",
  ...isi,
});
const KOSONG: DataBeranda = { beranda: null, layanan: null, untukPerusahaan: null, artikelTerbaru: [] };

describe("hero Beranda (C-03)", () => {
  test("tanpa gambar CMS: mockup ponsel dekoratif (aria-hidden), dua kartu mengambang faktual, CTA ganda, tiga chip nilai", () => {
    const { container } = render(<Beranda data={{ ...KOSONG, beranda: dasar({ title: "Halo", body: "Pembuka **singkat**.\n\n- butir satu\n- butir dua" }) }} />);
    const ponsel = container.querySelector(".situs-ponsel");
    expect(ponsel?.getAttribute("aria-hidden")).toBe("true");
    expect(ponsel?.textContent).toBe("");
    expect(container.querySelector(".situs-hero img")).toBeNull();
    expect(screen.getByText("Konsultasi chat dan video")).toBeDefined();
    expect(screen.getByText("Biaya MCU dan lab terlihat sebelum daftar")).toBeDefined();
    expect(screen.getByRole("link", { name: "Unduh aplikasi" }).getAttribute("href")).toBe("/unduh");
    expect(screen.getByRole("link", { name: "Untuk Perusahaan" }).getAttribute("href")).toBe("/untuk-perusahaan");
    const chip = screen.getByRole("list", { name: "Sorotan Ruang Sehat" });
    expect(within(chip).getAllByRole("listitem").map((li) => li.textContent)).toEqual([
      "Tujuh layanan terpadu",
      "Panggilan video tidak direkam",
      "Android & iOS",
    ]);
  });

  test("paragraf pembuka di hero; sisa isi CMS (daftar) tidak ditampilkan karena desain tidak punya pita itu (CR-042 lanjutan)", () => {
    const { container } = render(<Beranda data={{ ...KOSONG, beranda: dasar({ title: "Halo", body: "Pembuka **singkat**.\n\n- butir satu\n- butir dua" }) }} />);
    expect(container.querySelector(".situs-hero .situs-lead strong")?.textContent).toBe("singkat");
    expect(container.querySelector(".situs-hero ul.situs-chip-nilai")).not.toBeNull();
    expect(container.querySelector(".situs-pita")).toBeNull();
    expect(container.textContent).not.toContain("butir satu");
  });

  test("dengan gambar CMS: gambar di bingkai (bukan kotak putus-putus) dengan teks alternatif, tanpa mockup", () => {
    const { container } = render(<Beranda data={{ ...KOSONG, beranda: dasar({ image: { url: IMG, alt_text: "Ilustrasi aplikasi" } }) }} />);
    const img = screen.getByRole("img", { name: "Ilustrasi aplikasi" });
    expect(img.closest(".situs-bingkai")).not.toBeNull();
    expect(container.querySelector(".situs-ponsel")).toBeNull();
    expect(container.querySelector(".situs-gambar")).toBeNull();
  });

  test("gambar CMS tidak sah (domain luar atau tanpa alt) → mockup, bukan gambar", () => {
    for (const image of [
      { url: "https://contoh.test/a.png", alt_text: "x" },
      { url: IMG, alt_text: " " },
    ]) {
      const { container } = render(<Beranda data={{ ...KOSONG, beranda: dasar({ image }) }} />);
      expect(container.querySelector("img")).toBeNull();
      expect(container.querySelector(".situs-ponsel")).not.toBeNull();
      cleanup();
    }
  });

  test("semua SVG dekoratif aria-hidden dan tidak dapat difokus", () => {
    const { container } = render(<Beranda data={{ ...KOSONG, layanan: dasar({ body: LAYANAN_MD }) }} />);
    const svg = [...container.querySelectorAll("svg")];
    expect(svg.length).toBeGreaterThan(10);
    for (const s of svg) {
      const tersembunyi = s.closest('[aria-hidden="true"]') !== null;
      expect(tersembunyi, s.outerHTML.slice(0, 80)).toBe(true);
      expect(s.getAttribute("focusable")).toBe("false");
    }
  });
});

describe("kartu layanan di Beranda (C-02)", () => {
  test("format cocok: satu kartu per ##, urutan CMS, satu tautan per kartu ke halaman detail layanan", () => {
    const { container } = render(<Beranda data={{ ...KOSONG, layanan: dasar({ title: "Tujuh layanan", body: LAYANAN_MD }) }} />);
    const kartu = [...container.querySelectorAll(".situs-slider__daftar > li")];
    expect(kartu.map((k) => k.querySelector("h3")?.textContent)).toEqual(["Konsultasi dokter", "Home visit", "Layanan korporat", "Kolaborasi"]);
    for (const [i, k] of kartu.entries()) {
      const tautan = k.querySelectorAll("a");
      expect(tautan).toHaveLength(1);
      expect(tautan[0]?.getAttribute("href")).toBe(`/layanan/layanan-${String(i + 1)}`);
    }
    // Ringkasan berupa teks polos: tautan di Markdown tidak menjadi tautan bersarang di kartu.
    expect(kartu[0]?.querySelector("p")?.textContent).toBe("Chat dan video dengan dokter rekanan.");
    expect(kartu[2]?.classList.contains("situs-layanan--korporat")).toBe(true);
    expect(screen.getByText("Tujuh layanan").tagName).toBe("H2");
  });

  test("format tidak cocok: tanpa kartu, judul konten sebagai ringkasan dan tautan ke Layanan tetap ada", () => {
    const { container } = render(<Beranda data={{ ...KOSONG, layanan: dasar({ title: "Layanan kami", body: "# Satu\n\nTeks." }) }} />);
    expect(container.querySelector(".situs-slider")).toBeNull();
    expect(screen.getByText("Layanan kami")).toBeDefined();
    expect(screen.getByRole("link", { name: "Lihat semua layanan" }).getAttribute("href")).toBe("/layanan");
  });
});

describe("seksi Untuk Perusahaan di Beranda (seksi gelap, Q15)", () => {
  test("seksi gelap, judul CMS, paragraf pembuka, sorotan dari ## CMS (maks 4), satu tombol ke halaman perusahaan", () => {
    const body = "Pembuka korporat.\n\n## Vaksinasi\n\nA.\n\n## MCU\n\nB.\n\n## Lab\n\nC.\n\n## Privasi\n\nD.\n\n## Dokumen\n\nE.";
    const { container } = render(<Beranda data={{ ...KOSONG, untukPerusahaan: dasar({ title: "Layanan karyawan", body }) }} />);
    const seksi = container.querySelector("section.situs-sec--gelap");
    expect(seksi).not.toBeNull();
    if (seksi === null) return;
    expect(within(seksi as HTMLElement).getByRole("heading", { level: 2 }).textContent).toBe("Untuk Perusahaan");
    expect(seksi.textContent).toContain("Layanan karyawan");
    expect(seksi.textContent).toContain("Pembuka korporat.");
    const sorotan = within(seksi as HTMLElement).getByRole("list", { name: "Di halaman Untuk Perusahaan" });
    expect(within(sorotan).getAllByRole("listitem").map((li) => li.textContent)).toEqual(["Vaksinasi", "MCU", "Lab", "Privasi"]);
    expect(within(seksi as HTMLElement).getByRole("link", { name: "Pelajari layanan perusahaan" }).getAttribute("href")).toBe("/untuk-perusahaan");
  });
});

describe("halaman Layanan (C-02)", () => {
  test("format cocok: hero dengan pembuka, kartu h2 ber-id jangkar, ringkasan pendek (sisa isi hanya di halaman detail)", () => {
    const { container } = render(<HalamanLayanan konten={dasar({ title: "Tujuh layanan", body: LAYANAN_MD })} />);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Tujuh layanan");
    expect(container.querySelector(".situs-halhero .situs-lead")?.textContent).toBe("Semua layanan ada di aplikasi.");
    const kartu = [...container.querySelectorAll(".situs-grid-layanan--rinci > li")];
    expect(kartu.map((k) => k.id)).toEqual(["layanan-1", "layanan-2", "layanan-3", "layanan-4"]);
    expect(screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent)).toEqual(["Konsultasi dokter", "Home visit", "Layanan korporat", "Kolaborasi"]);
    // Kartu tidak memuat seluruh isi bagian: butir "Resep digital" dan teks di bawah ringkasan hanya di halaman detail.
    expect(kartu[0]?.querySelector("li")).toBeNull();
    expect(container.textContent).not.toContain("Resep digital");
    expect(kartu[0]?.querySelector(".situs-layanan__ringkas")?.textContent).toBe("Chat dan video dengan dokter rekanan.");
    expect(within(kartu[0] as HTMLElement).getByRole("link", { name: /Lihat detail layanan/ }).getAttribute("href")).toBe("/layanan/layanan-1");
    expect(kartu[3]?.querySelector(".situs-ikon-lingkar")).not.toBeNull();
  });

  test("format tidak cocok: Markdown biasa utuh (tanpa kartu), isi tidak hilang", () => {
    const { container } = render(<HalamanLayanan konten={dasar({ title: "Layanan", body: "# Bagian\n\nIsi **penting**." })} />);
    expect(container.querySelector(".situs-slider")).toBeNull();
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Bagian");
    expect(screen.getByText("penting").tagName).toBe("STRONG");
  });

  test("HTML mentah dan tautan berbahaya di bagian kartu tetap dinetralkan", () => {
    const body = "## Dokter\n\nRingkasan <script>alert(1)</script>.\n\n[x](javascript:alert(1)) <img src=x onerror=alert(1)>";
    const { container } = render(<HalamanLayanan konten={dasar({ body })} />);
    expect(container.querySelector("script")).toBeNull();
    expect(container.querySelector("img")).toBeNull();
    expect(container.querySelector('a[href^="javascript"]')).toBeNull();
    for (const el of container.querySelectorAll("*")) {
      for (const a of el.getAttributeNames()) expect(a.startsWith("on") || a === "style", `${el.tagName} ${a}`).toBe(false);
    }
  });
});

describe("halaman isi (C-01)", () => {
  test("pita judul terang; paragraf pembuka di pita, tiap ## menjadi bagian halaman (CR-042 lanjutan)", () => {
    const { container } = render(<HalamanKonten konten={dasar({ title: "Tentang", body: "Pembuka.\n\n## Siapa kami\n\nIsi." })} />);
    expect(container.querySelector(".situs-halhero .situs-lead")?.textContent).toBe("Pembuka.");
    expect(container.querySelector("section.situs-sec h2")?.textContent).toBe("Siapa kami");
  });

  test("Untuk Perusahaan (CR-042: tanpa varian gelap): pita judul terang dengan label", () => {
    const { container } = render(<HalamanKonten konten={dasar({ title: "Perusahaan", body: "Pembuka." })} label="Untuk HR dan perusahaan" />);
    const hero = container.querySelector(".situs-halhero");
    expect(hero).not.toBeNull();
    expect(container.querySelector("[class*='--gelap']")).toBeNull();
    expect(hero?.querySelector(".situs-eyebrow")?.textContent).toBe("Untuk HR dan perusahaan");
    expect(hero?.querySelector("h1")?.textContent).toBe("Perusahaan");
    // Isi hanya pembuka → tanpa kolom baca kosong.
    expect(container.querySelector(".situs-baca")).toBeNull();
  });
});

describe("tinjauan CR-033: draf dan masukan berbahaya", () => {
  test("Kebijakan (draf): paragraf pembuka tidak dipindah ke pita judul; spanduk Draf. sebelum teks isi pertama", () => {
    const { container } = render(<HalamanKonten konten={dasar({ title: "Kebijakan", body: "Paragraf pertama kebijakan.\n\n## Privasi\n\nIsi." })} draf />);
    expect(container.querySelector(".situs-halhero .situs-lead")).toBeNull();
    const catatan = screen.getByRole("note");
    const pertama = screen.getByText("Paragraf pertama kebijakan.");
    expect(catatan.compareDocumentPosition(pertama) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  test("Beranda: judul/ringkasan layanan dan sorotan korporat berbahaya → teks polos, tanpa elemen atau atribut aktif", () => {
    const layananMd = "## Dokter <img src=x onerror=alert(1)>\n\n[x](javascript:alert(1)) <script>alert(1)</script>\n\n## Lab\n\nAman.";
    const korporatMd = "Pembuka <script>alert(2)</script>.\n\n## Sorotan <img src=y onerror=alert(2)>\n\n[y](javascript:alert(2))";
    const { container } = render(<Beranda data={{ ...KOSONG, layanan: dasar({ body: layananMd }), untukPerusahaan: dasar({ body: korporatMd }) }} />);
    expect(container.querySelector("img, script, a[href^='javascript' i]")).toBeNull();
    for (const el of container.querySelectorAll("*")) {
      for (const a of el.getAttributeNames()) expect(a.startsWith("on") || a === "style", `${el.tagName} ${a}`).toBe(false);
    }
    const judul = [...container.querySelectorAll(".situs-slider__daftar h3")].map((h) => h.textContent);
    // teksPolos hanya mengambil node teks: tag HTML mentah sebaris dibuang (isi di antaranya tetap teks), tidak pernah elemen.
    expect(judul).toEqual(["Dokter", "Lab"]);
    expect(container.querySelector(".situs-slider__daftar p")?.textContent).toBe("x alert(1)");
    expect(screen.getByRole("list", { name: "Di halaman Untuk Perusahaan" }).textContent).toBe("Sorotan");
    expect(container.textContent).not.toContain("alert(2)</script>.");
  });
});
