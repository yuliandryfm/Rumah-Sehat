/**
 * Alamat halaman aplikasi di toko aplikasi (CR-038). Dibaca dari `SITUS_PLAYSTORE_URL` dan
 * `SITUS_APPSTORE_URL` per permintaan di server (bukan `NEXT_PUBLIC_*`). Aplikasi belum terbit sehingga
 * alamat toko `[TIDAK DIKETAHUI]`. Fail-closed seperti `tautan-korporat.ts`: nilai kosong atau tidak
 * valid → `null` (pemanggil mengarahkan ke halaman Unduh aplikasi), tanpa melempar galat.
 * Aturan: skema https saja, host persis `play.google.com` / `apps.apple.com` (huruf kecil persis seperti
 * diketik, tanpa port, tanpa userinfo, tanpa karakter kendali/spasi/backslash). Tidak ada pengecualian http.
 */
export type NamaToko = "playstore" | "appstore";

const ENV: Record<NamaToko, string> = { playstore: "SITUS_PLAYSTORE_URL", appstore: "SITUS_APPSTORE_URL" };
const HOST: Record<NamaToko, string> = { playstore: "play.google.com", appstore: "apps.apple.com" };

export function ambilUrlToko(toko: NamaToko, env: Record<string, string | undefined> = process.env): string | null {
  const mentah = env[ENV[toko]];
  if (mentah === undefined || mentah === "") return null;
  for (const ch of mentah) {
    const kode = ch.codePointAt(0) ?? 0;
    if (kode <= 0x20 || (kode >= 0x7f && kode <= 0x9f) || ch === "\\") return null;
  }
  // Skema dan host harus huruf kecil persis; parser URL menormalkan huruf besar sehingga diperiksa pada teks mentah.
  const cocok = /^https:\/\/([^/?#]*)/.exec(mentah);
  if (cocok === null) return null;
  let url: URL;
  try {
    url = new URL(mentah);
  } catch {
    return null;
  }
  if (cocok[1] !== HOST[toko] || url.hostname !== HOST[toko]) return null;
  if (url.protocol !== "https:" || url.username !== "" || url.password !== "" || url.port !== "") return null;
  return url.href;
}
