import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import type { NextConfig } from "next";
import { HEADER_STATIS, buatCsp } from "./src/keamanan.ts";

/**
 * Konfigurasi backoffice (ADR-024 butir 2, ADR-027 butir 5), pola sama dengan `web/situs`:
 * `output: "standalone"`, self-hosted; optimasi gambar mati; tanpa `X-Powered-By`; tanpa `next/og`, Edge
 * runtime, `use cache`, dan `next/image` (riwayat advisory, docs/05-stack.md §1.2). Header keamanan untuk
 * `/_next/static` dipasang di sini; semua rute lain dari `src/proxy.ts`.
 */
const akarWorkspace = dirname(dirname(fileURLToPath(import.meta.url)));

const konfigurasi: NextConfig = {
  output: "standalone",
  outputFileTracingRoot: akarWorkspace,
  poweredByHeader: false,
  reactStrictMode: true,
  images: { unoptimized: true },
  transpilePackages: ["@rs/ui", "@rs/bff-auth"],
  headers() {
    return Promise.resolve([
      {
        source: "/_next/static/:path*",
        headers: [...HEADER_STATIS, { key: "Content-Security-Policy", value: buatCsp(null) }],
      },
    ]);
  },
};

export default konfigurasi;
