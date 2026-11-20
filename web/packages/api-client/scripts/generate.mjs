// Pembangkit tipe klien API dari spesifikasi di openapi/ (ADR-022 butir 3; stack §1.2).
// Pemakaian: `node scripts/generate.mjs write` menulis src/generated/<nama>.ts;
//            `node scripts/generate.mjs check` gagal bila berkas di-commit berbeda dari hasil generate
//            atau ada spesifikasi tanpa hasil generate (dipanggil scripts/verify.sh).
// Hanya openapi/*.json yang dibangkitkan; openapi/asyncapi/ bukan OpenAPI dan tidak diproses.
import { execFileSync } from "node:child_process";
import { mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const paket = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const specDir = resolve(paket, "../../../openapi");
const outDir = join(paket, "src", "generated");
// `exports` openapi-typescript tidak membuka bin/; berkas ditemukan relatif terhadap package.json-nya.
const cli = join(dirname(createRequire(import.meta.url).resolve("openapi-typescript/package.json")), "bin", "cli.js");

const mode = process.argv[2];
if (mode !== "write" && mode !== "check") {
  console.error("pemakaian: generate.mjs write|check");
  process.exit(2);
}

const specs = readdirSync(specDir).filter((f) => f.endsWith(".json")).sort();
if (specs.length === 0) {
  console.error(`tidak ada spesifikasi *.json di ${specDir}`);
  process.exit(1);
}

const tmp = mkdtempSync(join(tmpdir(), "rs-api-client-"));
let status = 0;
try {
  for (const spec of specs) {
    const nama = basename(spec, ".json");
    // Argumen ditulis eksplisit tanpa shell.
    execFileSync(process.execPath, [cli, join(specDir, spec), "-o", join(tmp, `${nama}.ts`)], {
      stdio: ["ignore", "ignore", "inherit"],
    });
  }
  if (mode === "write") {
    mkdirSync(outDir, { recursive: true });
    for (const f of readdirSync(tmp)) {
      writeFileSync(join(outDir, f), readFileSync(join(tmp, f)));
    }
    console.log(`api-client: ${specs.length} berkas ditulis ke src/generated/`);
  } else {
    for (const f of readdirSync(tmp)) {
      const target = join(outDir, f);
      if (!existsSync(target) || !readFileSync(target).equals(readFileSync(join(tmp, f)))) {
        console.error(`src/generated/${f} berbeda dari hasil generate (atau belum ada); jalankan: pnpm generate`);
        status = 1;
      }
    }
    const dihasilkan = new Set(readdirSync(tmp));
    for (const f of existsSync(outDir) ? readdirSync(outDir) : []) {
      if (!dihasilkan.has(f)) {
        console.error(`src/generated/${f} tidak punya spesifikasi di openapi/ (hapus berkas)`);
        status = 1;
      }
    }
    if (status === 0) console.log(`api-client: src/generated/ identik dengan hasil generate (${specs.length} berkas)`);
  }
} finally {
  rmSync(tmp, { recursive: true, force: true });
}
process.exit(status);
