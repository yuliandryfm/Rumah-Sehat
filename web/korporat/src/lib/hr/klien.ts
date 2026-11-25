import { createKorporatClient, type KorporatComponents } from "@rs/api-client";

type S = KorporatComponents["schemas"];

export type AnggotaHr = S["HrMemberDto"];
export type StatusHr = S["HrStatusDto"];
export type HasilNonaktif = S["HrDeactivatedDto"];

export interface Gagal {
  readonly ok: false;
  readonly status: number;
  readonly kode: string | null;
  readonly pesan: string;
}
export type Hasil<T> = { readonly ok: true; readonly data: T } | Gagal;

/** Path di openapi/korporat.json sudah berawalan `/v1/corporate/`; dari browser dipanggil lewat `/bff`. */
export const BASIS_BFF = "/bff";

/** Kode 401 akun nonaktif (docs/05-response-codes.md, KU-10): hanya untuk pemegang sesi yang sah. */
export const KODE_AKUN_NONAKTIF = "4010002";
/** Kode 409 penonaktifan HR (openapi/korporat.json; docs/05-response-codes.md). */
export const KODE_HR_DIRI_SENDIRI = "4090101";
export const KODE_HR_AKTIF_TERAKHIR = "4090105";
export const KODE_SUDAH_NONAKTIF = "4090001";

/** Teks per `response_code` 409; dipilih dari kode, bukan dari isi respons mentah. */
const PESAN_KODE: Record<string, string> = {
  [KODE_HR_AKTIF_TERAKHIR]: "HR aktif terakhir tidak dapat dinonaktifkan.",
  [KODE_HR_DIRI_SENDIRI]: "Anda tidak dapat menonaktifkan akun Anda sendiri.",
  [KODE_SUDAH_NONAKTIF]: "Akun HR ini sudah nonaktif.",
};

const PESAN_STATUS: Record<number, string> = {
  401: "Sesi Anda berakhir. Masuk lagi untuk melanjutkan.",
  403: "Perusahaan Anda belum aktif, sehingga menu ini belum dapat dipakai.",
  404: "Data tidak ditemukan.",
  429: "Terlalu banyak permintaan. Coba lagi sebentar lagi.",
  503: "Layanan sedang tidak tersedia. Coba lagi nanti.",
};

/** Galat dari envelope RFC 9457 (ADR-023). Isi galat mentah tidak pernah ditampilkan atau dicatat. */
export function galat(status: number, badan: unknown): Gagal {
  const b = (typeof badan === "object" && badan !== null ? badan : {}) as Partial<S["ErrorEnvelope"]>;
  const kode = typeof b.response_code === "string" ? b.response_code : null;
  const pesan = (kode === null ? undefined : PESAN_KODE[kode]) ?? PESAN_STATUS[status] ?? "Permintaan gagal. Coba lagi.";
  return { ok: false, status, kode, pesan };
}

export interface KlienHr {
  daftar(): Promise<Hasil<readonly AnggotaHr[]>>;
  nonaktifkan(id: string): Promise<Hasil<HasilNonaktif>>;
}

export interface OpsiKlien {
  readonly basis?: string;
  readonly fetch?: typeof fetch;
}

interface Respons {
  readonly data?: unknown;
  readonly error?: unknown;
  readonly response: Response;
}

/** Klien dari browser lewat BFF; token CSRF sesi di `X-CSRF-Token` (token OIDC tidak ada di browser). */
export function buatKlienHr(csrfToken: string, opsi: OpsiKlien = {}): KlienHr {
  const fetchBff = opsi.fetch;
  const c = createKorporatClient({
    baseUrl: opsi.basis ?? BASIS_BFF,
    headers: { "x-csrf-token": csrfToken },
    ...(fetchBff === undefined ? {} : { fetch: (r: Request) => fetchBff(r) }),
  });
  const kerjakan = async <T>(p: Promise<Respons>, ambil: (data: unknown) => T): Promise<Hasil<T>> => {
    let r: Respons;
    try {
      r = await p;
    } catch {
      return galat(503, null);
    }
    if (!r.response.ok) return galat(r.response.status, r.error);
    const amplop = (typeof r.data === "object" && r.data !== null ? r.data : {}) as { data?: unknown };
    return { ok: true, data: ambil(amplop.data) };
  };
  return {
    daftar: () => kerjakan(c.GET("/v1/corporate/members"), (d) => (Array.isArray(d) ? (d as AnggotaHr[]) : [])),
    nonaktifkan: (id) =>
      kerjakan(
        c.POST("/v1/corporate/members/{corporate_member_id}/deactivate", { params: { path: { corporate_member_id: id } } }),
        (d) => d as HasilNonaktif,
      ),
  };
}
