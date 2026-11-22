/**
 * Daftar izin render aman isi CMS (SIT-01, ADR-027 butir 5, US-1802 AC6, TD-25). Port tepat dari validasi
 * backend `apps/rs-backend/src/modules/content_support/markdown.rs` (`allowed_link`, `allowed_image`,
 * `PUBLIC_IMAGE_PATH`): kedua sisi memakai aturan yang sama. Backend menolak saat simpan; situs tetap
 * menerapkan aturan ini saat render sebagai lapis kedua, jadi bagian yang melanggar tidak pernah menjadi
 * tautan atau gambar.
 */

/** Awalan jalur endpoint gambar konten publik (F1-T93); satu-satunya sumber gambar yang diizinkan. */
export const JALUR_GAMBAR_PUBLIK = "/v1/public/content/images/";

const POLA_UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const POLA_UUID_APA_SAJA = /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/;

/** UUID berbentuk baku (huruf besar/kecil), untuk ID di URL dan kursor sebelum dikirim ke backend. */
export function adalahUuid(nilai: string): boolean {
  return POLA_UUID_APA_SAJA.test(nilai);
}

// Karakter kendali (C0, DEL, C1) dan spasi Unicode: browser membuang tab/baris baru di dalam URL sehingga
// `java\tscript:` menjadi `javascript:`; semuanya ditolak, sama dengan `char::is_control`/`is_whitespace` Rust.
const POLA_KENDALI_ATAU_SPASI = /[\p{Cc}\s]/u;

/** Tujuan tautan yang diizinkan: `https`, `mailto`, atau relatif terhadap situs. */
export function tautanDiizinkan(tujuan: string): boolean {
  if (tujuan === "") return true;
  if (POLA_KENDALI_ATAU_SPASI.test(tujuan) || tujuan.includes("\\")) return false;
  if (tujuan.startsWith("//")) return false;
  // Skema = bagian sebelum ':' bila ':' muncul sebelum '/', '?', atau '#' (RFC 3986 §3, §4.2).
  const batas = tujuan.search(/[/?#]/);
  const kepala = batas === -1 ? tujuan : tujuan.slice(0, batas);
  const titikDua = kepala.indexOf(":");
  if (titikDua === -1) return true;
  const skema = tujuan.slice(0, titikDua).toLowerCase();
  const sisa = tujuan.slice(titikDua + 1);
  if (skema === "https") return sisa.startsWith("//") && sisa.length > 2;
  if (skema === "mailto") return sisa.length > 0;
  return false;
}

/** Sumber gambar yang diizinkan: tepat `/v1/public/content/images/<uuid huruf kecil>`. */
export function gambarDiizinkan(tujuan: string): boolean {
  return tujuan.startsWith(JALUR_GAMBAR_PUBLIK) && POLA_UUID.test(tujuan.slice(JALUR_GAMBAR_PUBLIK.length));
}
