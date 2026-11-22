import { createBffCore, createBffRoutes, loadBffConfig, type BffCore } from "@rs/bff-auth";

/**
 * Nama aplikasi BFF; harus sama dengan `BFF_APP_NAME` (nama cookie sesi `__Host-rs_korporat_sid`, awalan kunci
 * Valkey). Satu-satunya BFF yang boleh memegang token realm `korporat` (ADR-027 butir 1, ADR-025).
 */
export const BFF_APP = "korporat";

let core: Promise<BffCore> | null = null;

/**
 * Dibuat saat permintaan pertama, bukan saat modul dimuat (`next build` mengimpor modul rute tanpa env).
 * Kegagalan tidak di-cache supaya Valkey/OpenBao yang pulih bisa dipakai tanpa restart.
 */
export const getCore = (): Promise<BffCore> =>
  (core ??= createBffCore(loadBffConfig(process.env)).catch((e: unknown) => {
    core = null;
    throw e;
  }));

/**
 * Namespace API Rust kanal korporat (ADR-027 butir 2, KEPUTUSAN-251): `/v1/corporate/...` (dipanggil browser
 * sebagai `/bff/v1/corporate/...`). Path lain, termasuk namespace backoffice, portal, dan pasien, dijawab 404
 * tanpa diteruskan: sesi korporat tidak dapat memanggil API kanal lain. Batas sesi (idle 30 menit, mutlak
 * 12 jam, ADR-011 butir 7 dan ADR-027 butir 3) adalah kebijakan bawaan `SessionManager`.
 * `loginErrorPath` harus halaman publik: halaman terautentikasi mengalihkan ke login lagi (putaran).
 */
export const OPSI_ROUTES = {
  allowedApiPrefixes: ["v1/corporate/"],
  defaultReturnPath: "/",
  loginErrorPath: "/masuk-gagal",
  afterLogoutPath: "/",
} as const;

export const routes = createBffRoutes(getCore, OPSI_ROUTES);
