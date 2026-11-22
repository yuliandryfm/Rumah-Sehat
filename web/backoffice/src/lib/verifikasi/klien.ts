import { createBackofficeClient, type BackofficeComponents } from "@rs/api-client";
import type { AlasanTolak, Detail, FotoDokumen, ItemAntrean } from "./model.ts";

type S = BackofficeComponents["schemas"];

export interface Gagal {
  readonly ok: false;
  readonly status: number;
  readonly kode: string | null;
  readonly pesan: string;
}
export type Hasil<T> = { readonly ok: true; readonly data: T } | Gagal;

export const BASIS_BFF = "/bff";

/** Teks per `response_code` 409 (docs/05-response-codes.md; wireframe C02, US-201 AC5/AC7/AC8). */
const PESAN_KODE: Record<string, string> = {
  "4090203": "NIK ini sudah disetujui sebagai pemilik akun lain. Tolak pengajuan dengan alasan NIK tidak tersedia.",
  "4090204": "Pengajuan sudah diputuskan. Keputusan pertama yang tersimpan yang berlaku.",
  "4090205": "Pengajuan ini tidak dapat Anda putuskan (pemisahan tugas). Pengajuan diputuskan Verifikator lain.",
};

const PESAN_STATUS: Record<number, string> = {
  401: "Sesi Anda berakhir. Masuk lagi untuk melanjutkan.",
  403: "Akun Anda tidak memiliki sub-peran Verifikator.",
  404: "Pengajuan tidak ditemukan.",
  422: "Alasan atau keterangan tidak valid. Periksa isian lalu coba lagi.",
  429: "Terlalu banyak permintaan. Coba lagi sebentar lagi.",
  503: "Layanan sedang tidak tersedia. Coba lagi nanti.",
};

/**
 * Galat dari envelope RFC 9457 (ADR-023). Teks dipilih dari kode, bukan dari isi respons mentah, dan isi
 * galat tidak pernah dicatat ke console (bisa memuat data pasien).
 */
export function galat(status: number, badan: unknown): Gagal {
  const b = (typeof badan === "object" && badan !== null ? badan : {}) as Partial<S["ErrorEnvelope"]>;
  const kode = typeof b.response_code === "string" ? b.response_code : null;
  const pesan = (kode === null ? undefined : PESAN_KODE[kode]) ?? PESAN_STATUS[status] ?? "Permintaan gagal. Coba lagi.";
  return { ok: false, status, kode, pesan };
}

export interface Halaman {
  readonly items: readonly ItemAntrean[];
  readonly hasMore: boolean;
  readonly nextCursor: string | null;
}

export interface KlienVerifikasi {
  antrean(cursor?: string): Promise<Hasil<Halaman>>;
  alasan(): Promise<Hasil<readonly AlasanTolak[]>>;
  /** Membuka detail: E-05 tercatat di server dan NIK lengkap dikembalikan. */
  detail(id: string): Promise<Hasil<Detail>>;
  /** URL unduh berumur pendek; E-05 tercatat sebelum URL terbit. */
  fotoUrl(id: string): Promise<Hasil<FotoDokumen>>;
  setujui(id: string): Promise<Hasil<S["DecisionDto"]>>;
  tolak(id: string, kodeAlasan: string, keterangan: string): Promise<Hasil<S["DecisionDto"]>>;
}

interface Respons {
  readonly data?: unknown;
  readonly error?: unknown;
  readonly response: Response;
}

export interface OpsiKlien {
  readonly basis?: string;
  readonly fetch?: typeof fetch;
}

/** Klien dari browser lewat BFF; token CSRF sesi di `X-CSRF-Token` (token OIDC tidak ada di browser). */
export function buatKlienVerifikasi(csrfToken: string, opsi: OpsiKlien = {}): KlienVerifikasi {
  const fetchBff = opsi.fetch;
  const c = createBackofficeClient({
    baseUrl: opsi.basis ?? BASIS_BFF,
    headers: { "x-csrf-token": csrfToken },
    ...(fetchBff === undefined ? {} : { fetch: (r: Request) => fetchBff(r) }),
  });
  const kerjakan = async <T>(p: Promise<Respons>, ambil: (data: unknown, r: Respons) => T): Promise<Hasil<T>> => {
    let r: Respons;
    try {
      r = await p;
    } catch {
      return galat(503, null);
    }
    if (!r.response.ok) return galat(r.response.status, r.error);
    const amplop = (typeof r.data === "object" && r.data !== null ? r.data : {}) as { data?: unknown };
    return { ok: true, data: ambil(amplop.data, r) };
  };
  const path = (id: string) => ({ params: { path: { identity_verification_request_id: id } } });
  return {
    antrean: (cursor) =>
      kerjakan(
        c.GET("/backoffice/v1/identity-verifications", { params: { query: { limit: 20, ...(cursor === undefined ? {} : { cursor }) } } }),
        (d, r) => {
          const meta = (r.data as { meta?: S["MetaSchema"] }).meta ?? { has_more: false };
          return { items: (d ?? []) as ItemAntrean[], hasMore: meta.has_more, nextCursor: meta.next_cursor ?? null };
        },
      ),
    alasan: () => kerjakan(c.GET("/backoffice/v1/identity-verification-reasons"), (d) => (d ?? []) as AlasanTolak[]),
    detail: (id) => kerjakan(c.GET("/backoffice/v1/identity-verifications/{identity_verification_request_id}", path(id)), (d) => d as Detail),
    fotoUrl: (id) =>
      kerjakan(c.POST("/backoffice/v1/identity-verifications/{identity_verification_request_id}/document-photo-url", path(id)), (d) => d as FotoDokumen),
    setujui: (id) =>
      kerjakan(c.POST("/backoffice/v1/identity-verifications/{identity_verification_request_id}/approve", path(id)), (d) => d as S["DecisionDto"]),
    tolak: (id, kodeAlasan, keterangan) =>
      kerjakan(
        c.POST("/backoffice/v1/identity-verifications/{identity_verification_request_id}/reject", {
          ...path(id),
          body: { reason_code: kodeAlasan, ...(keterangan.trim() === "" ? {} : { note: keterangan.trim() }) },
        }),
        (d) => d as S["DecisionDto"],
      ),
  };
}
