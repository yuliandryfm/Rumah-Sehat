import { describe, expect, test } from "vitest";
import { BATAS_KARTU, ikonUntukJudul, pisahPembuka, uraiLayanan } from "./struktur.ts";

// Bentuk konten demo halaman Layanan (brief situs SP-03): pengantar + satu `##` per layanan.
const LAYANAN = [
  "Semua layanan dipesan di aplikasi.",
  "",
  "## Konsultasi dokter",
  "",
  "Chat dan video dengan **dokter** rekanan.",
  "",
  "- Resep digital",
  "- Video tidak direkam",
  "",
  "## Konsultasi psikolog",
  "",
  "Dukungan kesehatan mental.",
  "",
  "## Pendaftaran klinik",
  "",
  "Daftar ke klinik rekanan.",
  "",
  "## Pendaftaran lab",
  "",
  "Daftar pemeriksaan lab.",
  "",
  "## Home visit",
  "",
  "Tenaga medis datang ke rumah.",
  "",
  "## Informasi biaya MCU dan lab",
  "",
  "Lihat biaya sebelum mendaftar.",
  "",
  "## Belanja produk kesehatan",
  "",
  "Obat bebas dan produk kecantikan.",
  "",
  "## Layanan korporat",
  "",
  "Untuk perusahaan.",
].join("\n");

describe("uraiLayanan: format cocok", () => {
  test("setiap ## menjadi kartu, urutan sama dengan CMS, ringkasan = paragraf pertama (teks polos)", () => {
    const hasil = uraiLayanan(LAYANAN);
    expect(hasil).not.toBeNull();
    expect(hasil?.kartu.map((k) => k.judul)).toEqual([
      "Konsultasi dokter",
      "Konsultasi psikolog",
      "Pendaftaran klinik",
      "Pendaftaran lab",
      "Home visit",
      "Informasi biaya MCU dan lab",
      "Belanja produk kesehatan",
      "Layanan korporat",
    ]);
    expect(hasil?.kartu[0]?.ringkasan).toBe("Chat dan video dengan dokter rekanan.");
    expect(hasil?.kartu.map((k) => k.id)).toEqual(Array.from({ length: 8 }, (_, i) => `layanan-${String(i + 1)}`));
  });

  test("ikon dipetakan dari kata kunci judul", () => {
    expect(uraiLayanan(LAYANAN)?.kartu.map((k) => k.ikon)).toEqual(["konsultasi", "psikolog", "klinik", "lab", "home-visit", "biaya", "belanja", "korporat"]);
  });

  test("pengantar sebelum ## pertama dipisah; sisa bagian (daftar) disimpan tanpa paragraf ringkasan", () => {
    const hasil = uraiLayanan(LAYANAN);
    expect(hasil?.pengantar.map((n) => n.type)).toEqual(["paragraph"]);
    expect(hasil?.kartu[0]?.sisa.map((n) => n.type)).toEqual(["list"]);
    expect(hasil?.kartu[1]?.sisa).toEqual([]);
  });

  test("urutan berubah di CMS → urutan kartu ikut berubah (tidak ada urutan tetap di kode)", () => {
    const hasil = uraiLayanan("## Home visit\n\nA.\n\n## Konsultasi dokter\n\nB.");
    expect(hasil?.kartu.map((k) => k.judul)).toEqual(["Home visit", "Konsultasi dokter"]);
  });

  test("paragraf ringkasan tidak harus tepat setelah judul; ### di dalam bagian tetap bagian yang sama", () => {
    const hasil = uraiLayanan("## Pendaftaran lab\n\n- butir\n\nRingkasan lab.\n\n### Rincian\n\nTeks.");
    expect(hasil?.kartu).toHaveLength(1);
    expect(hasil?.kartu[0]?.ringkasan).toBe("Ringkasan lab.");
    expect(hasil?.kartu[0]?.sisa.map((n) => n.type)).toEqual(["list", "heading", "paragraph"]);
  });
});

describe("uraiLayanan: format tidak cocok → null (fallback Markdown biasa)", () => {
  test.each([
    ["isi null", null],
    ["isi kosong", "   "],
    ["tanpa ##", "Paragraf saja.\n\n- butir"],
    ["ada judul tingkat 1", "# Layanan\n\n## Dokter\n\nTeks."],
    ["bagian tanpa paragraf", "## Dokter\n\n- hanya daftar"],
    ["judul kosong", "##\n\nTeks."],
    ["ringkasan hanya spasi/kosong", "## Dokter\n\n![](/v1/public/content/images/0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b)"],
    ["terlalu banyak bagian", Array.from({ length: BATAS_KARTU + 1 }, (_, i) => `## L${String(i)}\n\nT.`).join("\n\n")],
  ])("%s", (_nama, teks) => {
    expect(uraiLayanan(teks)).toBeNull();
  });

  test("tepat BATAS_KARTU bagian masih cocok", () => {
    const teks = Array.from({ length: BATAS_KARTU }, (_, i) => `## L${String(i)}\n\nT.`).join("\n\n");
    expect(uraiLayanan(teks)?.kartu).toHaveLength(BATAS_KARTU);
  });
});

describe("ikonUntukJudul", () => {
  test.each([
    ["Konsultasi Psikolog", "psikolog"],
    ["Kesehatan mental", "psikolog"],
    ["Biaya MCU", "biaya"],
    ["Informasi biaya lab", "biaya"],
    ["Home visit dari klinik rekanan", "home-visit"],
    ["Untuk Perusahaan", "korporat"],
    ["Konsultasi online", "konsultasi"],
    ["Dokter umum", "konsultasi"],
    ["Klinik rekanan", "klinik"],
    ["Laboratorium", "lab"],
    ["Obat bebas", "belanja"],
    ["Produk kecantikan", "belanja"],
  ])("%s → %s", (judul, ikon) => {
    expect(ikonUntukJudul(judul)).toBe(ikon);
  });

  test.each(["Tentang kami", "Kolaborasi", "Labirin", ""])("tanpa kata kunci (%j) → ikon generik cadangan", (judul) => {
    expect(ikonUntukJudul(judul)).toBe("generik");
  });
});

describe("pisahPembuka", () => {
  test("paragraf pertama dipisah dari sisa", () => {
    const h = pisahPembuka("Pembuka.\n\n## Bagian\n\nIsi.");
    expect(h?.pembuka?.type).toBe("paragraph");
    expect(h?.sisa.map((n) => n.type)).toEqual(["heading", "paragraph"]);
  });

  test("isi diawali judul → tanpa pembuka, semua node menjadi sisa", () => {
    const h = pisahPembuka("# Judul\n\nTeks.");
    expect(h?.pembuka).toBeNull();
    expect(h?.sisa).toHaveLength(2);
  });

  test.each([null, undefined, "", "  \n"])("isi kosong (%j) → null", (teks) => {
    expect(pisahPembuka(teks)).toBeNull();
  });
});
