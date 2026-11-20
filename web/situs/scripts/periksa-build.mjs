// Pemeriksaan keluaran build situs (dijalankan setelah `next build` oleh skrip `build`, sehingga ikut
// langkah web-build di scripts/verify.sh). Menjalankan server standalone sungguhan dan memastikan:
// header keamanan dan CSP bernonce ada di setiap respons (halaman, 404, berkas statis), tidak ada
// Set-Cookie, tidak ada X-Powered-By, dan setiap <script> di HTML membawa nonce atau src berkas sendiri.
// Tanpa dependensi; gagal dengan kode keluar 1.
import { spawn } from "node:child_process";
import { cp, readFile, stat } from "node:fs/promises";
import { createServer } from "node:net";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const akarSitus = dirname(dirname(fileURLToPath(import.meta.url)));
const standalone = join(akarSitus, ".next", "standalone", "situs");
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

const WAJIB = {
  "strict-transport-security": /max-age=\d+/,
  "x-content-type-options": /^nosniff$/,
  "referrer-policy": /./,
  "permissions-policy": /camera=\(\)/,
  "content-security-policy": /frame-ancestors 'none'/,
};

function periksaHeader(nama, res) {
  for (const [kunci, pola] of Object.entries(WAJIB)) {
    const nilai = res.headers.get(kunci);
    periksa(nilai !== null && pola.test(nilai), `${nama}: header ${kunci} tidak ada atau salah (${nilai})`);
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
await cp(join(akarSitus, ".next", "static"), join(standalone, ".next", "static"), { recursive: true });

const port = await portBebas();
const server = spawn(process.execPath, ["server.js"], {
  cwd: standalone,
  env: {
    PATH: process.env["PATH"] ?? "",
    NODE_ENV: "production",
    PORT: String(port),
    HOSTNAME: "127.0.0.1",
    SITUS_KONTEN_SUMBER: "contoh",
  },
  stdio: ["ignore", "ignore", "inherit"],
});
const dasar = `http://127.0.0.1:${port}`;

try {
  await tunggu(`${dasar}/`, server);

  const beranda = await fetch(`${dasar}/`);
  periksa(beranda.status === 200, `/: status ${beranda.status}, seharusnya 200`);
  periksaHeader("/", beranda);
  const html = await beranda.text();
  const nonce = /'nonce-([^']+)'/.exec(beranda.headers.get("content-security-policy") ?? "")?.[1];
  periksa(nonce !== undefined, "/: CSP tanpa nonce");
  const skrip = [...html.matchAll(/<script\b([^>]*)>/g)].map((m) => m[1] ?? "");
  periksa(skrip.length > 0, "/: tidak ada <script> (runtime Next seharusnya ada)");
  for (const atribut of skrip) {
    periksa(nonce !== undefined && atribut.includes(`nonce="${nonce}"`), `/: <script> tanpa nonce yang cocok: ${atribut.slice(0, 80)}`);
  }
  periksa(!/\son[a-z]+\s*=/i.test(html), "/: ada atribut event handler inline");
  periksa(!/\sstyle\s*=/i.test(html), "/: ada atribut style inline");
  periksa(html.includes("[TIDAK DIKETAHUI]"), "/: penanda [TIDAK DIKETAHUI] tidak tampil");
  periksa(/<main[^>]*id="isi"/.test(html), "/: landmark <main id=isi> tidak ada");

  const nonce2 = /'nonce-([^']+)'/.exec((await fetch(`${dasar}/`)).headers.get("content-security-policy") ?? "")?.[1];
  periksa(nonce2 !== undefined && nonce2 !== nonce, "nonce tidak berganti antar permintaan");

  const hilang = await fetch(`${dasar}/halaman-yang-tidak-ada`);
  periksa(hilang.status === 404, `404: status ${hilang.status}`);
  periksaHeader("404", hilang);
  periksa((await hilang.text()).includes("Halaman tidak ditemukan"), "404: teks halaman tidak ditemukan tidak tampil");

  const css = /href="(\/_next\/static\/[^"]+\.css)"/.exec(html)?.[1];
  periksa(css !== undefined, "/: tautan CSS berkas statis tidak ditemukan");
  if (css !== undefined) {
    const res = await fetch(`${dasar}${css}`);
    periksa(res.status === 200, `statis: status ${res.status}`);
    periksaHeader("statis", res);
    periksa(!/https?:\/\//.test(await res.text()), "statis: CSS memuat URL jarak jauh");
  }
  periksa((await readFile(join(standalone, "server.js"), "utf8")).length > 0, "server.js kosong");
} catch (e) {
  gagal.push(String(e));
} finally {
  server.kill("SIGTERM");
}

if (gagal.length > 0) {
  console.error("periksa-build situs GAGAL:\n- " + gagal.join("\n- "));
  process.exit(1);
}
console.log("periksa-build situs: header keamanan, CSP bernonce, tanpa cookie, 404 dan berkas statis lulus");
