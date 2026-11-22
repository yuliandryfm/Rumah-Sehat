import createClient, { type ClientOptions } from "openapi-fetch";
import type { paths as BackofficePaths } from "./generated/backoffice.ts";
import type { paths as OpsPaths } from "./generated/ops.ts";

import type { paths as PublicPaths } from "./generated/public.ts";

export type { BackofficePaths };
export type { components as BackofficeComponents } from "./generated/backoffice.ts";
export type { OpsPaths, PublicPaths };
export type { components as PublicComponents } from "./generated/public.ts";
export type { components as OpsComponents } from "./generated/ops.ts";

/**
 * Klien bertipe untuk API operasional (`openapi/ops.json`). Field data tetap snake_case
 * apa adanya dari tipe hasil generate (KEPUTUSAN-119). Pemanggil dari komponen klien hanya
 * boleh memakai klien ini lewat Route Handler BFF (coding-standards §1.5).
 */
export function createOpsClient(options: ClientOptions) {
  return createClient<OpsPaths>(options);
}

/**
 * Klien bertipe untuk API backoffice (`openapi/backoffice.json`). Dari browser, `baseUrl` harus
 * `/bff` (Route Handler BFF) dan setiap permintaan mutasi membawa `X-CSRF-Token` (coding-standards §1.5).
 */
export function createBackofficeClient(options: ClientOptions) {
  return createClient<BackofficePaths>(options);
}

/** Klien bertipe untuk API publik tanpa sesi (`openapi/public.json`, `/v1/public/*`): dipakai situs publik. */
export function createPublicClient(options: ClientOptions) {
  return createClient<PublicPaths>(options);
}
