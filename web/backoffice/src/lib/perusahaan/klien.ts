import { createBackofficeClient } from "@rs/api-client";
import { galat, BASIS_BFF, type Gagal, type Hasil, type OpsiKlien } from "../verifikasi/klien.ts";
import type {
  AlasanTolakPerusahaan,
  DetailPerusahaan,
  DokumenUrl,
  ItemAntreanPerusahaan,
  JenisDokumen,
  KeputusanPerusahaan,
  PerusahaanDibuat,
  PermintaanBuatPerusahaan,
} from "./model.ts";

export type { Gagal, Hasil };

/** Kode 403 BFF untuk penolakan CSRF/Origin (`web/packages/bff-auth/src/http.ts`): bukan soal sub-peran. */
export const KODE_CSRF_DITOLAK = "4030101";

/** Kode 409 pembuatan perusahaan (`docs/05-response-codes.md`, US-1903 AC1/AC4). */
export const KODE_NIB_TERDAFTAR = "4090110";
export const KODE_EMAIL_HR_DIPAKAI = "4090111";
/** 409: persetujuan sebelum NIB, NPWP, dan bukti kewenangan lengkap (F1-T102). */
export const KODE_DOKUMEN_BELUM_LENGKAP = "4090113";
/** 409: dokumen berubah sejak detail dilihat; muat ulang lalu putuskan kembali (F1-T102). */
export const KODE_DOKUMEN_BERUBAH = "4090114";

/** Teks per `response_code` (409 `4090001` US-1902 AC4; 403 `4030101` dari BFF). */
const PESAN_KODE: Record<string, string> = {
  [KODE_CSRF_DITOLAK]: "Permintaan tidak dapat diverifikasi. Muat ulang halaman lalu coba lagi.",
  [KODE_NIB_TERDAFTAR]: "NIB sudah terdaftar. Hubungi Layanan Pelanggan bila perusahaan ini belum pernah didaftarkan.",
  [KODE_EMAIL_HR_DIPAKAI]: "Email HR sudah dipakai akun lain di portal perusahaan. Gunakan email HR yang lain.",
  "4090001": "Pengajuan sudah diputuskan Verifikator lain. Keputusan pertama yang tersimpan yang berlaku.",
  [KODE_DOKUMEN_BELUM_LENGKAP]: "Dokumen belum lengkap. NIB, NPWP, dan bukti kewenangan wajib ada sebelum disetujui.",
  [KODE_DOKUMEN_BERUBAH]: "Dokumen perusahaan berubah sejak Anda membukanya. Detail dimuat ulang; periksa dokumen lalu putuskan kembali.",
};

const PESAN_STATUS: Record<number, string> = {
  401: "Sesi Anda berakhir. Masuk lagi untuk melanjutkan.",
  403: "Akun Anda tidak memiliki sub-peran Verifikator.",
  404: "Perusahaan atau dokumen tidak ditemukan.",
  422: "Isian tidak valid. Periksa isian lalu coba lagi.",
  429: "Terlalu banyak permintaan. Coba lagi sebentar lagi.",
  503: "Layanan sedang tidak tersedia. Coba lagi nanti.",
};

/** Teks dipilih dari kode, bukan dari isi respons mentah (isi dan NPWP tidak pernah dicatat ke console). */
export function galatPerusahaan(status: number, badan: unknown): Gagal {
  const dasar = galat(status, badan);
  const pesan = (dasar.kode === null ? undefined : PESAN_KODE[dasar.kode]) ?? PESAN_STATUS[status] ?? "Permintaan gagal. Coba lagi.";
  return { ...dasar, pesan };
}

export interface HalamanPerusahaan {
  readonly items: readonly ItemAntreanPerusahaan[];
  readonly hasMore: boolean;
  readonly nextCursor: string | null;
}

export interface KlienPerusahaan {
  antrean(cursor?: string): Promise<Hasil<HalamanPerusahaan>>;
  alasan(): Promise<Hasil<readonly AlasanTolakPerusahaan[]>>;
  /** Membuka detail (NIB dan NPWP lengkap untuk Verifikator). */
  detail(id: string): Promise<Hasil<DetailPerusahaan>>;
  /** URL unduh berumur pendek; E-05 tercatat di server sebelum URL terbit. */
  dokumenUrl(id: string, jenis: JenisDokumen): Promise<Hasil<DokumenUrl>>;
  /** `versiDokumen` = `documents_version` dari detail yang dilihat Verifikator (keputusan terikat versi dokumen). */
  setujui(id: string, versiDokumen: string): Promise<Hasil<KeputusanPerusahaan>>;
  tolak(id: string, kodeAlasan: string, versiDokumen: string): Promise<Hasil<KeputusanPerusahaan>>;
  /** Membuat perusahaan Aktif dan mengundang HR pertama (US-1903). Isian tidak pernah dicatat ke console. */
  buat(isian: PermintaanBuatPerusahaan): Promise<Hasil<PerusahaanDibuat>>;
}

interface Respons {
  readonly data?: unknown;
  readonly error?: unknown;
  readonly response: Response;
}

/** Klien dari browser lewat BFF; token CSRF sesi di `X-CSRF-Token` (token OIDC tidak ada di browser). */
export function buatKlienPerusahaan(csrfToken: string, opsi: OpsiKlien = {}): KlienPerusahaan {
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
      return galatPerusahaan(503, null);
    }
    if (!r.response.ok) return galatPerusahaan(r.response.status, r.error);
    const amplop = (typeof r.data === "object" && r.data !== null ? r.data : {}) as { data?: unknown };
    return { ok: true, data: ambil(amplop.data, r) };
  };
  const path = (id: string) => ({ params: { path: { corporate_client_id: id } } });
  return {
    antrean: (cursor) =>
      kerjakan(
        c.GET("/backoffice/v1/corporate-verifications", { params: { query: { limit: 20, ...(cursor === undefined ? {} : { cursor }) } } }),
        (d, r) => {
          const meta = (r.data as { meta?: { has_more: boolean; next_cursor?: string | null } }).meta ?? { has_more: false };
          return { items: (d ?? []) as ItemAntreanPerusahaan[], hasMore: meta.has_more, nextCursor: meta.next_cursor ?? null };
        },
      ),
    alasan: () => kerjakan(c.GET("/backoffice/v1/corporate-verification-reasons"), (d) => (d ?? []) as AlasanTolakPerusahaan[]),
    detail: (id) => kerjakan(c.GET("/backoffice/v1/corporate-verifications/{corporate_client_id}", path(id)), (d) => d as DetailPerusahaan),
    dokumenUrl: (id, jenis) =>
      kerjakan(
        c.POST("/backoffice/v1/corporate-verifications/{corporate_client_id}/documents/{document_kind}/download-url", {
          params: { path: { corporate_client_id: id, document_kind: jenis } },
        }),
        (d) => d as DokumenUrl,
      ),
    setujui: (id, versiDokumen) =>
      kerjakan(
        c.POST("/backoffice/v1/corporate-verifications/{corporate_client_id}/approve", {
          ...path(id),
          body: { documents_version: versiDokumen },
        }),
        (d) => d as KeputusanPerusahaan,
      ),
    tolak: (id, kodeAlasan, versiDokumen) =>
      kerjakan(
        c.POST("/backoffice/v1/corporate-verifications/{corporate_client_id}/reject", {
          ...path(id),
          body: { reason_code: kodeAlasan, documents_version: versiDokumen },
        }),
        (d) => d as KeputusanPerusahaan,
      ),
    buat: async (isian) => {
      const h = await kerjakan(
        c.POST("/backoffice/v1/corporate-clients", {
          body: {
            legal_name: isian.legal_name.trim(),
            nib: isian.nib.trim(),
            npwp: isian.npwp.trim(),
            office_address: isian.office_address.trim(),
            office_phone: isian.office_phone.trim(),
            hr_email: isian.hr_email.trim(),
          },
        }),
        (d) => d as PerusahaanDibuat,
      );
      return h;
    },
  };
}
