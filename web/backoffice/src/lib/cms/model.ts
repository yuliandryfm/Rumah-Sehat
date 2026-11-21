import type { BackofficeComponents } from "@rs/api-client";

type S = BackofficeComponents["schemas"];
export type Konten = S["ContentDto"];
export type Kanal = S["Channel"];
export type JenisKonten = S["ContentType"];
export type StatusTampil = S["DisplayStatus"];
export type HalamanSitus = S["SitePage"];
export type GalatIsian = S["FieldErrorSchema"];
export type PermintaanKonten = S["ContentRequest"];

/** Label tampilan (wireframe C14). Nama status final mengikuti PRD §3.6 `[TIDAK DIKETAHUI]` bila berbeda. */
export const LABEL_STATUS: Record<StatusTampil, string> = {
  draft: "Draf",
  scheduled: "Tayang (terjadwal)",
  live: "Tayang",
  ended: "Selesai",
  archived: "Diarsipkan",
  deleted: "Terhapus",
};

/** Nada badge per status (design guideline §5.6): Draf netral, Tayang sukses. */
export const NADA_STATUS: Record<StatusTampil, "neutral" | "info" | "success" | "warning" | "danger"> = {
  draft: "neutral",
  scheduled: "info",
  live: "success",
  ended: "neutral",
  archived: "warning",
  deleted: "danger",
};

export const LABEL_KANAL: Record<Kanal, string> = {
  app: "Aplikasi",
  site: "Situs",
  app_and_site: "Aplikasi dan situs",
};

export const LABEL_JENIS: Record<JenisKonten, string> = {
  banner: "Banner",
  info_page: "Halaman info",
  article: "Artikel",
  promo: "Promo",
};

/** Nama halaman situs mengikuti wireframe C14 (BR-130). */
export const LABEL_HALAMAN: Record<HalamanSitus, string> = {
  home: "Beranda",
  about: "Tentang Kami",
  services: "Layanan",
  for_business: "Untuk Perusahaan",
  partners: "Rekanan",
  articles: "Artikel/berita",
  contact: "Kontak",
  download_app: "Unduh aplikasi",
  privacy_terms: "Kebijakan privasi & syarat",
};

export const URUTAN_STATUS: readonly StatusTampil[] = ["draft", "scheduled", "live", "ended", "archived", "deleted"];
export const URUTAN_KANAL: readonly Kanal[] = ["app", "site", "app_and_site"];
export const URUTAN_JENIS: readonly JenisKonten[] = ["banner", "info_page", "article", "promo"];
export const URUTAN_HALAMAN: readonly HalamanSitus[] = [
  "home",
  "about",
  "services",
  "for_business",
  "partners",
  "articles",
  "contact",
  "download_app",
  "privacy_terms",
];

/** Draf adalah satu-satunya status yang boleh diubah (backend menjawab 409 untuk lainnya). */
export const bolehDiubah = (k: Pick<Konten, "display_status">): boolean => k.display_status === "draft";
export const bolehDijadwalkan = bolehDiubah;
export const bolehDiarsipkan = (k: Pick<Konten, "display_status">): boolean =>
  k.display_status === "draft" || k.display_status === "scheduled";
export const bolehDihapus = (k: Pick<Konten, "display_status">): boolean => k.display_status !== "deleted";

/** Hasil format dipakai sebagai teks (React meng-escape); tidak pernah dimasukkan ke HTML mentah. */
export function formatWaktu(iso: string | null | undefined): string {
  if (iso === null || iso === undefined) return "-";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "-";
  return d.toLocaleString("id-ID", { dateStyle: "medium", timeStyle: "short", timeZone: "Asia/Jakarta" });
}
