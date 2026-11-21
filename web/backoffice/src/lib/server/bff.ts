import { createBffCore, createBffRoutes, loadBffConfig, type BffCore } from "@rs/bff-auth";

/** Nama aplikasi BFF; harus sama dengan `BFF_APP_NAME` (nama cookie sesi `__Host-rs_backoffice_sid`). */
export const BFF_APP = "backoffice";

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
 * Namespace API Rust untuk kanal admin (ADR-006 butir 5): `/backoffice/v1/...` sesuai `openapi/backoffice.json`
 * (dipanggil browser sebagai `/bff/backoffice/v1/...`); path lain dijawab 404 tanpa diteruskan.
 * `loginErrorPath` harus halaman publik: halaman terautentikasi mengalihkan ke login lagi (putaran).
 */
export const OPSI_ROUTES = {
  allowedApiPrefixes: ["backoffice/v1/"],
  defaultReturnPath: "/",
  loginErrorPath: "/masuk-gagal",
  afterLogoutPath: "/",
} as const;

export const routes = createBffRoutes(getCore, OPSI_ROUTES);
