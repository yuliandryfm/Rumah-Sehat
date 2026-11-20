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

/** Namespace API Rust untuk kanal admin (ADR-006 butir 5); path lain dijawab 404 tanpa diteruskan. */
export const routes = createBffRoutes(getCore, {
  allowedApiPrefixes: ["v1/admin/"],
  defaultReturnPath: "/",
  loginErrorPath: "/",
  afterLogoutPath: "/",
});
