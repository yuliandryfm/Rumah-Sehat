// Pemeriksaan keluaran build backoffice (dijalankan setelah `next build` oleh skrip `build`, sehingga ikut
// langkah web-build di scripts/verify.sh). Menjalankan server standalone sungguhan TANPA env BFF_* dan
// memastikan: header keamanan, CSP bernonce, dan X-Correlation-Id ada di setiap respons (halaman, 404, rute
// BFF, berkas statis); tidak ada Set-Cookie dan X-Powered-By; setiap <script> membawa nonce; halaman
// terautentikasi tanpa cookie dialihkan ke login tanpa menyentuh Valkey; rute BFF terpasang dan gagal
// tertutup (503) tanpa konfigurasi. Tanpa dependensi; gagal dengan kode keluar 1.
import { spawn } from "node:child_process";
import { cp, readFile, stat } from "node:fs/promises";
import { createServer } from "node:net";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const akarApp = dirname(dirname(fileURLToPath(import.meta.url)));
const standalone = join(akarApp, ".next", "standalone", "backoffice");
const gagal = [];
const periksa = (kondisi, pesan) => {
  if (!kondisi) gagal.push(pesan);
};

async function portBebas() {
  return await new Promise((resolve, reject) => {
    const s = createServer();
    s.once("error", reject);
    s.listen(0, "127.0.0.1", () => {
      const { port } = s.address();
      s.close(() => resolve(port));
    });
  });
}

async function tunggu(url, server) {
  for (let i = 0; i < 100; i++) {
    if (server.exitCode !== null) throw new Error(`server standalone berhenti (kode ${server.exitCode})`);
    try {
      await fetch(url);
      return;
    } catch {
      await new Promise((r) => setTimeout(r, 100));
    }
  }
  throw new Error("server standalone tidak siap dalam 10 detik");
}

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

const WAJIB = {
  "strict-transport-security": /max-age=\d+/,
  "x-content-type-options": /^nosniff$/,
  "referrer-policy": /./,
  "permissions-policy": /camera=\(\)/,
  "content-security-policy": /frame-ancestors 'none'/,
};

function periksaHeader(nama, res, { korelasi = true } = {}) {
  for (const [kunci, pola] of Object.entries(WAJIB)) {
    const nilai = res.headers.get(kunci);
    periksa(nilai !== null && pola.test(nilai), `${nama}: header ${kunci} tidak ada atau salah (${nilai})`);
  }
  if (korelasi) {
    const id = res.headers.get("x-correlation-id") ?? "";
    periksa(UUID.test(id), `${nama}: X-Correlation-Id bukan UUID (${id})`);
  }
  periksa(res.headers.getSetCookie().length === 0, `${nama}: ada Set-Cookie`);
  periksa(res.headers.get("x-powered-by") === null, `${nama}: ada X-Powered-By`);
  const csp = res.headers.get("content-security-policy") ?? "";
  periksa(!/unsafe-inline|unsafe-eval/.test(csp), `${nama}: CSP memuat unsafe-*`);
  periksa(!/https?:\/\//.test(csp), `${nama}: CSP memuat host pihak ketiga`);
}

await stat(join(standalone, "server.js")).catch(() => {
  console.error(`periksa-build: ${standalone}/server.js tidak ada (output standalone gagal)`);
  process.exit(1);
});
// Server standalone tidak menyalin berkas statis sendiri (docs Next.js, output: standalone).
await cp(join(akarApp, ".next", "static"), join(standalone, ".next", "static"), { recursive: true });

const port = await portBebas();
const server = spawn(process.execPath, ["server.js"], {
  cwd: standalone,
  env: {
    PATH: process.env["PATH"] ?? "",
    NODE_ENV: "production",
    PORT: String(port),
    HOSTNAME: "127.0.0.1",
  },
  stdio: ["ignore", "ignore", "inherit"],
});
const dasar = `http://127.0.0.1:${port}`;

try {
  await tunggu(`${dasar}/akses-ditolak`, server);

  const ditolak = await fetch(`${dasar}/akses-ditolak`);
  periksa(ditolak.status === 200, `/akses-ditolak: status ${ditolak.status}`);
  periksaHeader("/akses-ditolak", ditolak);
  const html = await ditolak.text();
  const nonce = /'nonce-([^']+)'/.exec(ditolak.headers.get("content-security-policy") ?? "")?.[1];
  periksa(nonce !== undefined, "/akses-ditolak: CSP tanpa nonce");
  const skrip = [...html.matchAll(/<script\b([^>]*)>/g)].map((m) => m[1] ?? "");
  periksa(skrip.length > 0, "/akses-ditolak: tidak ada <script> (runtime Next seharusnya ada)");
  for (const atribut of skrip) {
    periksa(nonce !== undefined && atribut.includes(`nonce="${nonce}"`), `<script> tanpa nonce yang cocok: ${atribut.slice(0, 80)}`);
  }
  periksa(!/\son[a-z]+\s*=/i.test(html), "/akses-ditolak: ada atribut event handler inline");
  periksa(!/\sstyle\s*=/i.test(html), "/akses-ditolak: ada atribut style inline");
  periksa(/<main[^>]*id="isi"/.test(html), "/akses-ditolak: landmark <main id=isi> tidak ada");
  periksa(html.includes("Data tidak ditemukan"), "/akses-ditolak: teks tidak tampil");
  const idHeader = ditolak.headers.get("x-correlation-id") ?? "";
  periksa(html.replaceAll("<!-- -->", "").includes(`Kode referensi: ${idHeader}`), "/akses-ditolak: kode referensi tidak sama dengan X-Correlation-Id");

  const dikirim = "0193a5e6-7f00-7abc-8def-0123456789ab";
  const gema = await fetch(`${dasar}/akses-ditolak`, { headers: { "x-correlation-id": dikirim } });
  periksa(gema.headers.get("x-correlation-id") === dikirim, "X-Correlation-Id UUID sah dari klien tidak dipakai ulang");
  const palsu = await fetch(`${dasar}/akses-ditolak`, { headers: { "x-correlation-id": "<script>x</script>" } });
  periksa(UUID.test(palsu.headers.get("x-correlation-id") ?? ""), "X-Correlation-Id tidak sah tidak diganti UUID baru");
  const nonce2 = /'nonce-([^']+)'/.exec(gema.headers.get("content-security-policy") ?? "")?.[1];
  periksa(nonce2 !== undefined && nonce2 !== nonce, "nonce tidak berganti antar permintaan");

  const nonaktif = await fetch(`${dasar}/akun-nonaktif`);
  periksa(nonaktif.status === 200, `/akun-nonaktif: status ${nonaktif.status}`);
  periksaHeader("/akun-nonaktif", nonaktif);
  periksa((await nonaktif.text()).includes("Hubungi admin Ruang Sehat"), "/akun-nonaktif: teks tidak tampil");

  const gagalMasuk = await fetch(`${dasar}/masuk-gagal?auth_error=expired`);
  periksa(gagalMasuk.status === 200, `/masuk-gagal: status ${gagalMasuk.status}`);
  periksaHeader("/masuk-gagal", gagalMasuk);

  const hilang = await fetch(`${dasar}/halaman-yang-tidak-ada`);
  periksa(hilang.status === 404, `404: status ${hilang.status}`);
  periksaHeader("404", hilang);
  const teks404 = await hilang.text();
  periksa(teks404.includes("Data tidak ditemukan") && teks404.includes("Kode referensi"), "404: pesan dan kode referensi tidak tampil");

  // Halaman terautentikasi tanpa cookie sesi: dialihkan ke login tanpa menyentuh Valkey; tanpa isi halaman.
  const beranda = await fetch(`${dasar}/`, { redirect: "manual" });
  periksa([302, 303, 307].includes(beranda.status), `/ tanpa sesi: status ${beranda.status}, seharusnya pengalihan`);
  periksa(
    /^(https?:\/\/[^/]+)?\/auth\/login$/.test(beranda.headers.get("location") ?? ""),
    `/ tanpa sesi: Location ${beranda.headers.get("location")}`,
  );
  periksaHeader("/ tanpa sesi", beranda);

  // Rute BFF terpasang dan gagal tertutup tanpa konfigurasi (tanpa env BFF_*): 503, no-store, tanpa Set-Cookie.
  for (const [metode, jalur] of [
    ["GET", "/auth/session"],
    ["GET", "/auth/login"],
    ["GET", "/auth/callback"],
    ["POST", "/auth/logout"],
    ["POST", "/auth/session/extend"],
    ["GET", "/bff/v1/admin/apa-saja"],
  ]) {
    const res = await fetch(`${dasar}${jalur}`, { method: metode, redirect: "manual" });
    periksa(res.status === 503, `${metode} ${jalur}: status ${res.status}, seharusnya 503 (gagal tertutup)`);
    periksa(res.headers.get("cache-control") === "no-store", `${metode} ${jalur}: bukan no-store`);
    periksaHeader(`${metode} ${jalur}`, res);
    periksa(!/BFF_|valkey|openbao/i.test(await res.text()), `${metode} ${jalur}: badan membocorkan nama konfigurasi`);
  }
  const salahMetode = await fetch(`${dasar}/auth/logout`, { method: "GET" });
  periksa(salahMetode.status === 405, `GET /auth/logout: status ${salahMetode.status}, seharusnya 405`);

  const css = /href="(\/_next\/static\/[^"]+\.css)"/.exec(html)?.[1];
  periksa(css !== undefined, "/akses-ditolak: tautan CSS berkas statis tidak ditemukan");
  if (css !== undefined) {
    const res = await fetch(`${dasar}${css}`);
    periksa(res.status === 200, `statis: status ${res.status}`);
    periksaHeader("statis", res, { korelasi: false });
    periksa(!/https?:\/\//.test(await res.text()), "statis: CSS memuat URL jarak jauh");
  }
  periksa((await readFile(join(standalone, "server.js"), "utf8")).length > 0, "server.js kosong");
} catch (e) {
  gagal.push(String(e));
} finally {
  server.kill("SIGTERM");
}

if (gagal.length > 0) {
  console.error("periksa-build backoffice GAGAL:\n- " + gagal.join("\n- "));
  process.exit(1);
}
console.log("periksa-build backoffice: header keamanan, CSP bernonce, kode referensi, pengalihan login, rute BFF gagal tertutup, 404 dan berkas statis lulus");
