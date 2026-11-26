/**
 * Alamat member area korporat (CR-032). Dibaca dari `SITUS_KORPORAT_URL` per permintaan di server
 * (bukan `NEXT_PUBLIC_*`, tidak dibakukan saat build), sehingga satu image dapat dipakai untuk domain berbeda.
 * Domain member area `[TIDAK DIKETAHUI]`. Fail-closed: nilai tidak valid atau kosong → `null` (tautan tidak
 * dirender), tanpa melempar galat.
 */
export function ambilUrlKorporat(env: Record<string, string | undefined> = process.env): string | null {
  const mentah = env["SITUS_KORPORAT_URL"];
  if (mentah === undefined || mentah === "") return null;
  // Tolak karakter kendali, spasi, dan backslash sebelum diurai (parser URL menormalkan/memangkasnya).
  for (const ch of mentah) {
    const kode = ch.codePointAt(0) ?? 0;
    if (kode <= 0x20 || (kode >= 0x7f && kode <= 0x9f) || ch === "\\") return null;
  }
  let url: URL;
  try {
    url = new URL(mentah);
  } catch {
    return null;
  }
  if (url.username !== "" || url.password !== "" || url.hostname === "") return null;
  if (url.protocol === "https:") return url.href;
  if (url.protocol === "http:" && env["NODE_ENV"] === "development" && (url.hostname === "localhost" || url.hostname === "127.0.0.1")) {
    return url.href;
  }
  return null;
}
