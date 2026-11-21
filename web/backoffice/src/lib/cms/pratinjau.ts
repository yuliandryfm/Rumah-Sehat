/**
 * Dokumen pratinjau untuk `<iframe sandbox srcdoc>` (US-1802 AC7, AC8).
 *
 * TODO(F1-T96): renderer Markdown yang sama dengan `web/situs` belum ada di branch ini (menunggu merge
 * F1-T96 dan keputusan lisensi). Sementara, Markdown ditampilkan sebagai TEKS POLOS di `<pre>`: seluruh
 * isi di-escape, jadi payload HTML/skrip tidak pernah menjadi markup. Setelah F1-T96 digabung, ganti
 * isi `<pre>` dengan keluaran render situs; atribut sandbox (tanpa `allow-same-origin`, tanpa
 * `allow-scripts`) dan escape judul tidak berubah.
 */

export function escapeHtml(teks: string): string {
  return teks
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

/** Sandbox paling ketat: nilai kosong = semua pembatasan (tanpa skrip, tanpa same-origin, tanpa form/popup). */
export const SANDBOX_PRATINJAU = "";

export function dokumenPratinjau(judul: string, markdown: string): string {
  return [
    "<!doctype html>",
    '<html lang="id"><head><meta charset="utf-8">',
    '<meta name="referrer" content="no-referrer">',
    "<title>Pratinjau</title></head><body>",
    `<h1>${escapeHtml(judul)}</h1>`,
    `<pre>${escapeHtml(markdown)}</pre>`,
    "</body></html>",
  ].join("");
}
