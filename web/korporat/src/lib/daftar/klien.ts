import { createPublicClient, type PublicComponents } from "@rs/api-client";
import { badanPermintaan, type Isian } from "./model.ts";

type S = PublicComponents["schemas"];

export type TeksPersetujuan = S["RegistrationConsentTextDto"];
export type Diterima = S["CorporateRegistrationAcceptedDto"];

/** Kode 409 versi teks persetujuan basi (docs/05-response-codes.md). */
export const KODE_TEKS_BERUBAH = "4090106";

export interface GagalDaftar {
  readonly ok: false;
  readonly status: number;
  readonly kode: string | null;
  readonly pesan: string;
  /** Galat per isian dari 422 (`errors[]`): `field` → pesan server (Bahasa Indonesia, ADR-023). */
  readonly isian: Readonly<Record<string, string>>;
}
export type HasilDaftar<T> = { readonly ok: true; readonly data: T } | GagalDaftar;

const PESAN_STATUS: Record<number, string> = {
  409: "Teks persetujuan telah berubah. Baca teks terbaru di bawah, lalu setujui kembali.",
  422: "Ada isian yang perlu diperbaiki.",
  429: "Terlalu banyak permintaan. Coba lagi beberapa saat lagi.",
  503: "Pendaftaran sedang tidak dapat diproses. Coba lagi nanti.",
};
const PESAN_UMUM = "Pendaftaran belum terkirim. Isian Anda masih ada; coba lagi.";
const PESAN_TERHUBUNG = "Gagal terhubung. Pendaftaran belum terkirim dan isian Anda masih ada. Periksa koneksi lalu coba lagi.";

/** Galat dari envelope RFC 9457. Isi `response_description` mentah tidak ditampilkan; pesan per isian dari server. */
export function galatDaftar(status: number, badan: unknown): GagalDaftar {
  const b = (typeof badan === "object" && badan !== null ? badan : {}) as Partial<S["ErrorEnvelope"]>;
  const kode = typeof b.response_code === "string" ? b.response_code : null;
  const isian: Record<string, string> = {};
  if (status === 422 && Array.isArray(b.errors)) {
    for (const e of b.errors) {
      if (typeof e.field === "string" && typeof e.message === "string" && !(e.field in isian)) isian[e.field] = e.message;
    }
  }
  const pesan = status === 409 && kode !== KODE_TEKS_BERUBAH ? PESAN_UMUM : (PESAN_STATUS[status] ?? PESAN_UMUM);
  return { ok: false, status, kode, pesan, isian };
}

const gagalTerhubung = (): GagalDaftar => ({ ok: false, status: 0, kode: null, pesan: PESAN_TERHUBUNG, isian: {} });

export interface KlienDaftar {
  teks(): Promise<HasilDaftar<TeksPersetujuan>>;
  kirim(isian: Isian, versiTeks: string, butir: readonly string[]): Promise<HasilDaftar<Diterima>>;
}

export interface OpsiKlienDaftar {
  /** Bawaan: asal halaman yang sama (Caddy host korporat meneruskan `/v1/public/corporate-registrations*`). */
  readonly basis?: string;
  readonly fetch?: typeof fetch;
  /** Batas klien (KU-02: 15 detik). */
  readonly batasMs?: number;
}

interface Respons {
  readonly data?: unknown;
  readonly error?: unknown;
  readonly response: Response;
}

/**
 * Klien pendaftaran publik dari browser (Q30 = C, KEPUTUSAN-372): langsung ke API publik di asal web korporat yang
 * sama, BUKAN lewat BFF. Tanpa cookie (`credentials: "omit"`), tanpa token CSRF, tanpa sesi: endpoint publik tidak
 * memakai otoritas ambien, jadi tidak ada yang dapat dipalsukan lintas situs (ADR-012 amandemen KEPUTUSAN-323).
 */
export function buatKlienDaftar(opsi: OpsiKlienDaftar = {}): KlienDaftar {
  const fetchKustom = opsi.fetch;
  const batas = opsi.batasMs ?? 15_000;
  const c = createPublicClient({
    baseUrl: opsi.basis ?? (typeof window === "undefined" ? "" : window.location.origin),
    credentials: "omit",
    ...(fetchKustom === undefined ? {} : { fetch: (r: Request) => fetchKustom(r) }),
  });
  const kerjakan = async <T>(p: () => Promise<Respons>): Promise<HasilDaftar<T>> => {
    let r: Respons;
    try {
      r = await p();
    } catch {
      return gagalTerhubung();
    }
    if (!r.response.ok) return galatDaftar(r.response.status, r.error);
    const amplop = (typeof r.data === "object" && r.data !== null ? r.data : {}) as { data?: unknown };
    return { ok: true, data: amplop.data as T };
  };
  return {
    teks: () =>
      kerjakan<TeksPersetujuan>(() =>
        c.GET("/v1/public/corporate-registrations/consent-text", { signal: AbortSignal.timeout(batas) }),
      ),
    kirim: (isian, versiTeks, butir) =>
      kerjakan<Diterima>(() =>
        c.POST("/v1/public/corporate-registrations", {
          body: badanPermintaan(isian, versiTeks, butir),
          signal: AbortSignal.timeout(batas),
        }),
      ),
  };
}
