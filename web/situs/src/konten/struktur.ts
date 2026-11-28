import { Parser, type Node } from "commonmark";

/**
 * Konvensi struktur Markdown CMS untuk tampilan situs (CR-033 C-02, KEPUTUSAN-354). Pengurai yang sama dengan
 * `Markdown.tsx` (commonmark.js tanpa ekstensi); keluarannya node pohon, tidak pernah string HTML, sehingga
 * render tetap lewat `NodeMd` yang aman. Bila struktur tidak cocok, fungsi mengembalikan `null` dan pemanggil
 * memakai render Markdown biasa (fallback aman, tidak ada isi yang hilang atau dikarang).
 */

/** Nama ikon layanan (SVG buatan sendiri di `komponen/Ikon.tsx`). */
export type NamaIkonLayanan = "konsultasi" | "psikolog" | "klinik" | "lab" | "home-visit" | "biaya" | "belanja" | "korporat" | "generik";

/**
 * Pemetaan kata kunci judul → ikon. Urutan penting: yang lebih spesifik dulu ("Konsultasi psikolog" →
 * psikolog, "Informasi biaya MCU dan lab" → biaya, "Home visit … klinik" → home visit). Judul yang tidak
 * cocok memakai ikon generik (daun), bukan ikon tebakan.
 */
const PETA_IKON: readonly (readonly [RegExp, NamaIkonLayanan])[] = [
  [/psikolog|kesehatan mental/, "psikolog"],
  [/\bmcu\b|biaya/, "biaya"],
  [/home visit|kunjungan rumah/, "home-visit"],
  [/perusahaan|korporat|karyawan/, "korporat"],
  [/konsultasi/, "konsultasi"],
  [/klinik/, "klinik"],
  [/\blab\b|laboratorium/, "lab"],
  [/obat|produk|belanja|apotek|kecantikan/, "belanja"],
  [/dokter|tenaga kesehatan/, "konsultasi"],
];

export function ikonUntukJudul(judul: string): NamaIkonLayanan {
  const kecil = judul.toLowerCase();
  for (const [pola, ikon] of PETA_IKON) if (pola.test(kecil)) return ikon;
  return "generik";
}

/** Kelompok tab filter penggeser layanan di Beranda (desain Stitch `beranda_ruang_sehat`, KEPUTUSAN-490). */
export type KategoriLayanan = "konsultasi" | "klinik-lab" | "home-belanja";

export const TAB_KATEGORI: readonly { id: KategoriLayanan; label: string }[] = [
  { id: "konsultasi", label: "Konsultasi" },
  { id: "klinik-lab", label: "Klinik & Lab" },
  { id: "home-belanja", label: "Home & Belanja" },
];

/**
 * Kelompok kartu menurut ikon hasil `ikonUntukJudul` (model konten CMS belum punya kolom kategori). Pengelompokan
 * dari desain user: Konsultasi = dokter + psikolog; Klinik & Lab = pendaftaran klinik, lab, informasi biaya MCU/lab;
 * Home & Belanja = home visit + belanja produk. Kartu lain (korporat, generik) hanya tampil di tab "Semua".
 */
export function kategoriLayanan(ikon: NamaIkonLayanan): KategoriLayanan | null {
  switch (ikon) {
    case "konsultasi":
    case "psikolog":
      return "konsultasi";
    case "klinik":
    case "lab":
    case "biaya":
      return "klinik-lab";
    case "home-visit":
    case "belanja":
      return "home-belanja";
    default:
      return null;
  }
}

/** Teks polos seluruh isi node (judul kartu, ringkasan, teks alternatif gambar). */
export function teksPolos(node: Node): string {
  let s = "";
  const jalan = node.walker();
  for (let e = jalan.next(); e !== null; e = jalan.next()) {
    if (e.entering && (e.node.type === "text" || e.node.type === "code")) s += e.node.literal ?? "";
    if (e.entering && (e.node.type === "softbreak" || e.node.type === "linebreak")) s += " ";
  }
  return s;
}

function anakTingkatAtas(teks: string): Node[] {
  const dokumen = new Parser().parse(teks);
  const hasil: Node[] = [];
  for (let n = dokumen.firstChild; n !== null; n = n.next) hasil.push(n);
  return hasil;
}

/** Foto kartu: hanya jalur lokal situs (CSP `img-src 'self'`), teks alternatif Indonesia wajib. */
export interface GambarKartu {
  src: string;
  alt: string;
}

export interface KartuLayanan {
  /** ID jangkar buatan kode (`layanan-1`, …), bukan dari konten. */
  id: string;
  judul: string;
  /** Paragraf pertama bagian sebagai teks polos (kartu Beranda seluruhnya tautan; tanpa tautan bersarang). */
  ringkasan: string;
  ikon: NamaIkonLayanan;
  /** Slot foto (opsional): terisi bila berkas foto lokal sudah ada (TD-69); selama kosong kartu memakai panel ikon. */
  gambar?: GambarKartu;
  /** Node bagian selain judul dan paragraf ringkasan (dirender di kartu rinci halaman Layanan). */
  sisa: Node[];
}

/** Kartu tanpa node Markdown (`sisa`): bentuk yang dapat diserialisasi ke komponen klien. */
export function tanpaSisa(k: KartuLayanan): Omit<KartuLayanan, "sisa"> {
  return { id: k.id, judul: k.judul, ringkasan: k.ringkasan, ikon: k.ikon, ...(k.gambar === undefined ? {} : { gambar: k.gambar }) };
}

export interface UraianLayanan {
  /** Node sebelum `##` pertama. */
  pengantar: Node[];
  kartu: KartuLayanan[];
}

/** Batas kartu: di atas ini struktur dianggap bukan daftar layanan (fallback ke Markdown biasa). */
export const BATAS_KARTU = 24;

/**
 * Mengurai Markdown halaman `services`: setiap `##` = satu kartu, paragraf pertama di bagian itu = ringkasan,
 * urutan = urutan di CMS (PRD US-1801 AC2). Mengembalikan `null` (fallback) bila: isi kosong, ada judul `#`
 * (tingkat 1), tidak ada `##`, lebih dari `BATAS_KARTU` bagian, judul kosong, atau bagian tanpa paragraf
 * berisi teks.
 */
export function uraiLayanan(teks: string | null | undefined): UraianLayanan | null {
  if (teks === null || teks === undefined || teks.trim() === "") return null;
  const pengantar: Node[] = [];
  const bagian: { judul: Node; isi: Node[] }[] = [];
  for (const n of anakTingkatAtas(teks)) {
    if (n.type === "heading" && n.level === 1) return null;
    if (n.type === "heading" && n.level === 2) {
      bagian.push({ judul: n, isi: [] });
      continue;
    }
    const kini = bagian.at(-1);
    if (kini === undefined) pengantar.push(n);
    else kini.isi.push(n);
  }
  if (bagian.length === 0 || bagian.length > BATAS_KARTU) return null;

  const kartu: KartuLayanan[] = [];
  for (const [i, b] of bagian.entries()) {
    const judul = teksPolos(b.judul).replace(/\s+/g, " ").trim();
    const posisi = b.isi.findIndex((n) => n.type === "paragraph");
    const paragraf = posisi < 0 ? undefined : b.isi[posisi];
    const ringkasan = paragraf === undefined ? "" : teksPolos(paragraf).replace(/\s+/g, " ").trim();
    if (judul === "" || ringkasan === "") return null;
    kartu.push({
      id: `layanan-${String(i + 1)}`,
      judul,
      ringkasan,
      ikon: ikonUntukJudul(judul),
      sisa: b.isi.filter((_, j) => j !== posisi),
    });
  }
  return { pengantar, kartu };
}

/**
 * Memisah paragraf pembuka (node tingkat atas pertama, bila paragraf) dari sisa isi, untuk kalimat pembuka di
 * hero halaman (CR-033 C-01/C-03). Isi kosong → `null`.
 */
export function pisahPembuka(teks: string | null | undefined): { pembuka: Node | null; sisa: Node[] } | null {
  if (teks === null || teks === undefined || teks.trim() === "") return null;
  const semua = anakTingkatAtas(teks);
  const pertama = semua[0];
  if (pertama?.type === "paragraph") return { pembuka: pertama, sisa: semua.slice(1) };
  return { pembuka: null, sisa: semua };
}
