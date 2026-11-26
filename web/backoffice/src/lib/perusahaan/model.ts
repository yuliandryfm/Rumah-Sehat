import type { BackofficeComponents } from "@rs/api-client";

type S = BackofficeComponents["schemas"];

/**
 * Bentuk respons antrean dan keputusan verifikasi perusahaan (F1-T104, `corporate/dto.rs`).
 * Dua DTO ini ditulis lokal karena `openapi/backoffice.json` memberi nama skema yang sama
 * (`QueueItemDto`, `DecisionDto`, `QueueEnvelope`, `DecisionEnvelope`) dengan modul verifikasi identitas, sehingga
 * tipe hasil generator untuk endpoint perusahaan menunjuk ke bentuk identitas (cacat spesifikasi, dilaporkan).
 * Skema yang tidak bentrok (`DetailDto`, `DocumentUrlDto`, `DocumentKindDto`, `CorporateStatusDto`) tetap dari generator.
 */
export type OriginPerusahaan = S["OriginDto"];
export type StatusPerusahaan = S["CorporateStatusDto"];
export type JenisDokumen = S["DocumentKindDto"];
export type DetailPerusahaan = S["DetailDto"];
export type DokumenUrl = S["DocumentUrlDto"];
export type AlasanTolakPerusahaan = S["RejectionReasonDto"];
export type PermintaanBuatPerusahaan = S["CorporateClientCreateRequest"];
export type PerusahaanDibuat = S["CorporateClientCreatedDto"];

export interface ItemAntreanPerusahaan {
  readonly corporate_client_id: string;
  readonly legal_name: string;
  readonly nib: string;
  readonly origin: OriginPerusahaan;
  readonly submission_round: number;
  readonly submitted_at: string;
}

export interface KeputusanPerusahaan {
  readonly corporate_client_id: string;
  readonly status: StatusPerusahaan;
  readonly submission_round: number;
  readonly rejection_reason?: string | null;
}

export const LABEL_DOKUMEN_PERUSAHAAN: Record<JenisDokumen, string> = {
  nib: "Dokumen NIB",
  npwp: "Dokumen NPWP",
  authority_proof: "Bukti kewenangan pendaftar",
};

export const LABEL_STATUS_PERUSAHAAN: Record<StatusPerusahaan, string> = {
  pending_verification: "Menunggu verifikasi",
  active: "Aktif",
  rejected: "Ditolak",
  inactive: "Nonaktif",
};

export const LABEL_ORIGIN: Record<OriginPerusahaan, string> = {
  self_registered: "Mendaftar sendiri",
  created_by_admin: "Dibuat admin",
};

/** Kolom "Sumber" wireframe C13: putaran > 1 berarti pengajuan ulang setelah ditolak. */
export function sumber(item: Pick<ItemAntreanPerusahaan, "origin" | "submission_round">): string {
  return item.submission_round > 1 ? "Pengajuan ulang" : LABEL_ORIGIN[item.origin];
}

/** NIB di daftar: hanya empat digit terakhir (wireframe C13 `************1234`); lengkap hanya di panel detail. */
export function nibTermasking(nib: string): string {
  return `************${nib.slice(-4)}`;
}

/** Batas panjang isian sesuai kontrak `CorporateClientCreateRequest` (`openapi/backoffice.json`). */
export const BATAS_FORM = { legal_name: 200, nib: 64, npwp: 64, office_address: 500, office_phone: 32, hr_email: 254 } as const;

export type KolomForm = keyof PermintaanBuatPerusahaan;

/**
 * Validasi bantuan di klien (server tetap penentu, 422). Format resmi NIB/NPWP belum ditetapkan, jadi hanya
 * memeriksa terisi, tanpa spasi, dan batas panjang; email hanya bentuk dasar.
 */
export function validasiForm(f: PermintaanBuatPerusahaan): Partial<Record<KolomForm, string>> {
  const e: Partial<Record<KolomForm, string>> = {};
  const wajib: Record<KolomForm, string> = {
    legal_name: "Nama badan usaha",
    nib: "NIB",
    npwp: "NPWP",
    office_address: "Alamat kantor",
    office_phone: "Telepon kantor",
    hr_email: "Email HR pertama",
  };
  for (const k of Object.keys(wajib) as KolomForm[]) {
    const v = f[k].trim();
    if (v === "") e[k] = `${wajib[k]} wajib diisi.`;
    else if (v.length > BATAS_FORM[k]) e[k] = `${wajib[k]} maksimal ${String(BATAS_FORM[k])} karakter.`;
  }
  for (const k of ["nib", "npwp"] as const) {
    if (e[k] === undefined && /\s/.test(f[k].trim())) e[k] = `${wajib[k]} tidak boleh mengandung spasi.`;
  }
  if (e.hr_email === undefined && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(f.hr_email.trim())) {
    e.hr_email = "Masukkan email yang valid, misalnya nama@perusahaan.example.";
  }
  return e;
}
