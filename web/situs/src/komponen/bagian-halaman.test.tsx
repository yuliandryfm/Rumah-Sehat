import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { uraiLayanan } from "../konten/struktur.ts";
import type { Konten } from "../konten/tipe.ts";
import { HalamanKonten } from "./HalamanKonten.tsx";
import { DetailLayanan } from "./Layanan.tsx";

vi.mock("next/navigation", () => ({ usePathname: () => "/" }));
afterEach(cleanup);

const konten = (body: string): Konten => ({
  content_id: "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b",
  content_type: "info_page",
  title: "Judul halaman",
  body,
  published_at: "2026-11-20T00:00:00Z",
});

describe("BagianHalaman (KEPUTUSAN-492)", () => {
  test("## menjadi bagian bertajuk h2 dengan latar bergantian; ### menjadi kartu h3", () => {
    const md = "Pembuka.\n\n## Prinsip\n\nPengantar.\n\n### Satu\n\nIsi satu.\n\n### Dua\n\nIsi dua.\n\n## Penutup\n\nTeks biasa.";
    const { container } = render(<HalamanKonten konten={konten(md)} />);
    const bagian = [...container.querySelectorAll("section.situs-sec")];
    expect(bagian.map((b) => b.querySelector("h2")?.textContent)).toEqual(["Prinsip", "Penutup"]);
    expect(bagian[0]?.classList.contains("situs-sec--putih")).toBe(false);
    expect(bagian[1]?.classList.contains("situs-sec--putih")).toBe(true);
    expect([...container.querySelectorAll(".situs-kartu-info h3")].map((h) => h.textContent)).toEqual(["Satu", "Dua"]);
    expect(bagian[0]?.getAttribute("aria-labelledby")).toBe(bagian[0]?.querySelector("h2")?.id);
  });

  test("bagian bernama cara/langkah memberi kartu bernomor; bagian pertanyaan menjadi akordeon details", () => {
    const md = "## Cara kerjanya\n\n### Daftar\n\nA.\n\n### Pilih\n\nB.\n\n## Hal yang sering ditanyakan\n\n### Apa ini?\n\nJawaban.";
    const { container } = render(<HalamanKonten konten={konten(md)} />);
    expect([...container.querySelectorAll(".situs-nomor")].map((n) => n.textContent)).toEqual(["1", "2"]);
    const d = container.querySelector("details.situs-tanya");
    expect(d?.querySelector("summary")?.textContent).toBe("Apa ini?");
    expect(d?.textContent).toContain("Jawaban.");
  });

  test('susunan="kartu": setiap ## menjadi satu kartu dalam satu grid (pola Kontak)', () => {
    const md = "## Pengguna aplikasi\n\nLewat aplikasi.\n\n## Perusahaan\n\nLewat portal.\n\n## Calon rekanan\n\nIsi formulir.";
    const { container } = render(<HalamanKonten konten={konten(md)} susunan="kartu" />);
    expect(container.querySelectorAll("ul.situs-kartu-grid")).toHaveLength(1);
    expect([...container.querySelectorAll(".situs-kartu-info h2")].map((h) => h.textContent)).toEqual(["Pengguna aplikasi", "Perusahaan", "Calon rekanan"]);
  });

  test("halaman tanpa ## tetap kolom baca biasa; HTML mentah tetap teks (tanpa elemen aktif)", () => {
    const { container } = render(<HalamanKonten konten={konten("Pembuka.\n\nParagraf <script>alert(1)</script> dua.")} />);
    expect(container.querySelector("script")).toBeNull();
    expect(container.querySelector(".situs-baca, section")).not.toBeNull();
    expect(container.textContent).toContain("alert(1)");
  });

  test("draf (Kebijakan) tidak memakai bagian bergaya: kolom baca dengan spanduk draf", () => {
    const { container } = render(<HalamanKonten konten={konten("## Pasal\n\nIsi.")} draf />);
    expect(screen.getByRole("note")).toBeDefined();
    expect(container.querySelector("section.situs-sec")).toBeNull();
  });
});

describe("DetailLayanan (desain detail_layanan, KEPUTUSAN-492)", () => {
  const md = "## Konsultasi dokter\n\nChat dan video.\n\n### Cara memakai\n\n#### Pilih\n\nA.\n\n#### Konsultasi\n\nB.\n\n### Pertanyaan umum\n\n#### Apakah privat?\n\nYa.";
  const kartu = uraiLayanan(md)?.kartu[0];

  test("pita judul dari kartu, bagian ### menjadi bagian halaman dengan langkah bernomor dan akordeon, tautan kembali", () => {
    if (kartu === undefined) throw new Error("kartu tidak ada");
    const { container } = render(<DetailLayanan kartu={kartu} />);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Konsultasi dokter");
    expect(container.querySelector(".situs-halhero .situs-lead")?.textContent).toBe("Chat dan video.");
    expect([...container.querySelectorAll("section h2")].map((h) => h.textContent)).toEqual(["Cara memakai", "Pertanyaan umum"]);
    expect([...container.querySelectorAll(".situs-nomor")].map((n) => n.textContent)).toEqual(["1", "2"]);
    expect(container.querySelector("details summary")?.textContent).toBe("Apakah privat?");
    expect(within(container).getByRole("link", { name: "Semua layanan" }).getAttribute("href")).toBe("/layanan");
  });
});
