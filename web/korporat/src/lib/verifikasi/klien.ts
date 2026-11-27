import { createKorporatClient, type KorporatComponents } from "@rs/api-client";
import { BASIS_BFF, KODE_CSRF_DITOLAK, galat as galatUmum, type Gagal, type Hasil } from "../hr/klien.ts";

type S = KorporatComponents["schemas"];

export type Ringkasan = S["DocumentsOverviewDto"];
export type ProfilPerusahaan = S["CompanyProfileDto"];
export type StatusDokumen = S["DocumentStatusDto"];
export type JenisDokumen = S["DocumentKindDto"];
export type StatusVerifikasi = S["VerificationStatusDto"];
export type UrlUnduh = S["DocumentUrlDto"];

/** Kode 409 (openapi/korporat.json; docs/05-response-codes.md). */
export const KODE_SUDAH_AKTIF = "4090112";
export const KODE_DOKUMEN_BELUM_LENGKAP = "4090115";
export const KODE_SUDAH_MENUNGGU = "4090001";

/**
 * Teks per `response_code` untuk layar dokumen dan pengajuan ulang. Dipisah dari teks Kelola HR karena kode
 * `4090001` di sana berarti "HR sudah nonaktif"; di sini "pengajuan sudah menunggu verifikasi".
 */
const PESAN_KODE: Record<string, string> = {
  [KODE_SUDAH_AKTIF]: "Perusahaan Anda sudah Aktif. Dokumen tidak dapat diubah lagi; muat ulang halaman untuk melihat status terbaru.",
  [KODE_DOKUMEN_BELUM_LENGKAP]: "Dokumen belum lengkap. Unggah Dokumen NIB, Dokumen NPWP, dan Bukti kewenangan pendaftar, lalu ajukan ulang.",
  [KODE_SUDAH_MENUNGGU]: "Pengajuan Anda sudah menunggu verifikasi, jadi tidak perlu diajukan ulang. Muat ulang halaman untuk melihat status terbaru.",
  [KODE_CSRF_DITOLAK]: "Permintaan tidak dapat diverifikasi. Muat ulang halaman lalu coba lagi.",
};

const PESAN_STATUS: Record<number, string> = {
  404: "Waktu unggah habis atau berkas tidak ditemukan. Pilih berkas lalu unggah lagi.",
  413: "Ukuran berkas melebihi batas. Foto JPG atau PNG paling besar 5 MiB, PDF paling besar 10 MiB.",
  // Q12 A: PDF bertanda tangan digital ditolak; selama penetralan PDF belum aktif di server (TD-48) semua PDF ditolak 415.
  415: "Berkas tidak dapat diterima. PDF bertanda tangan digital, terenkripsi, atau rusak ditolak, dan PDF mungkin belum dapat diterima saat ini. Unggah hasil pindaian atau foto dokumen dalam format JPG atau PNG.",
  422: "Berkas belum terunggah dengan benar. Pilih berkas lalu unggah lagi.",
  429: "Terlalu banyak unggahan dalam waktu singkat. Coba lagi beberapa saat lagi.",
};

/** Galat dari envelope RFC 9457: kode dan status dari respons, teks dari tabel di atas lalu tabel umum. */
export function galat(status: number, badan: unknown): Gagal {
  const g = galatUmum(status, badan);
  const pesan = (g.kode === null ? undefined : PESAN_KODE[g.kode]) ?? PESAN_STATUS[status] ?? g.pesan;
  return { ...g, pesan };
}

/** Gagal terhubung (KU-02): permintaan tidak sampai atau tidak selesai; isian dan pilihan berkas tetap ada. */
export const gagalTerhubung = (): Gagal => ({
  ok: false,
  status: 0,
  kode: null,
  pesan: "Gagal terhubung. Periksa koneksi Anda lalu coba lagi.",
});

/** Hasil memuat: ringkasan (Menunggu verifikasi/Ditolak) atau perusahaan sudah Aktif (409 `4090112`). */
export type Muatan = { readonly jenis: "dokumen"; readonly ringkasan: Ringkasan } | { readonly jenis: "aktif" };

export interface KlienVerifikasi {
  muat(): Promise<Hasil<Muatan>>;
  /** Slot unggah → unggah langsung ke object storage → konfirmasi. Mengembalikan status dokumen terbaru. */
  unggah(jenis: JenisDokumen, berkas: File): Promise<Hasil<Ringkasan>>;
  urlUnduh(jenis: JenisDokumen): Promise<Hasil<UrlUnduh>>;
  ajukanUlang(): Promise<Hasil<Ringkasan>>;
  /**
   * Koreksi data perusahaan sekaligus ajukan ulang (CR-036; satu aksi di server). `badan` hanya berisi field
   * yang dikoreksi. Galat 422 membawa pesan server per field di `isian`.
   */
  koreksiDanAjukanUlang(badan: BadanKoreksi): Promise<Hasil<Ringkasan> | GagalKoreksi>;
}

export type BadanKoreksi = S["CompanyCorrectionRequest"];

/** Galat koreksi: seperti [`Gagal`] ditambah pesan per field dari 422 (`errors[]`, ADR-023). */
export interface GagalKoreksi extends Gagal {
  readonly isian: Readonly<Record<string, string>>;
}

/** Galat koreksi dari envelope RFC 9457: pesan per field dari server, teks umum dari tabel layar ini. */
export function galatKoreksi(status: number, badan: unknown): GagalKoreksi {
  const g = galat(status, badan);
  const b = (typeof badan === "object" && badan !== null ? badan : {}) as { errors?: unknown };
  const isian: Record<string, string> = {};
  if (Array.isArray(b.errors)) {
    for (const e of b.errors as { field?: unknown; message?: unknown }[]) {
      if (typeof e.field === "string" && typeof e.message === "string" && !(e.field in isian)) isian[e.field] = e.message;
    }
  }
  const pesan =
    status === 422
      ? "Ada data perusahaan yang perlu diperbaiki. Periksa pesan di setiap isian."
      : status === 429
        ? "Terlalu banyak percobaan pengajuan ulang. Coba lagi beberapa saat lagi; isian Anda masih ada."
        : g.pesan;
  return { ...g, pesan, isian };
}

export interface OpsiKlienVerifikasi {
  readonly basis?: string;
  readonly fetch?: typeof fetch;
  /** Fetch ke URL bertanda tangan object storage (bukan BFF). */
  readonly fetchUnggah?: typeof fetch;
  /** Batas unggah langsung (KU-02: 60 detik). */
  readonly batasUnggahMs?: number;
}

interface Respons {
  readonly data?: unknown;
  readonly error?: unknown;
  readonly response: Response;
}

/**
 * Klien dari browser lewat BFF (`/bff/v1/corporate/...`, token CSRF sesi di `X-CSRF-Token`). Unggahan berkas
 * mengikuti pola unggah gambar CMS backoffice (F1-T95, ADR-018): isi berkas langsung ke URL bertanda tangan object
 * storage tanpa cookie dan tanpa referrer, lalu dikonfirmasi lewat BFF; server memeriksa isi berkas saat konfirmasi.
 */
export function buatKlienVerifikasi(csrfToken: string, opsi: OpsiKlienVerifikasi = {}): KlienVerifikasi {
  const fetchUnggah: typeof fetch = opsi.fetchUnggah ?? ((...a) => fetch(...a));
  const batasUnggah = opsi.batasUnggahMs ?? 60_000;
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
      return gagalTerhubung();
    }
    if (!r.response.ok) return galat(r.response.status, r.error);
    const amplop = (typeof r.data === "object" && r.data !== null ? r.data : {}) as { data?: unknown };
    return { ok: true, data: ambil(amplop.data) };
  };
  const ringkasan = (d: unknown) => d as Ringkasan;
  return {
    muat: async () => {
      const h = await kerjakan(c.GET("/v1/corporate/documents"), ringkasan);
      if (h.ok) return { ok: true, data: { jenis: "dokumen", ringkasan: h.data } };
      if (h.status === 409 && h.kode === KODE_SUDAH_AKTIF) return { ok: true, data: { jenis: "aktif" } };
      return h;
    },
    unggah: async (jenis, berkas) => {
      const slot = await kerjakan(
        c.POST("/v1/corporate/document-upload-slots", { body: { content_type: berkas.type, size_bytes: berkas.size } }),
        (d) => d as S["CorporateUploadSlotDto"],
      );
      if (!slot.ok) return slot;
      try {
        // URL bertanda tangan adalah rahasia berumur pendek: tidak dicatat dan tidak ditampilkan.
        const s = slot.data;
        const sinyal = AbortSignal.timeout(batasUnggah);
        let res: Response;
        if (s.method === "post") {
          const f = new FormData();
          for (const p of s.fields) f.append(p.name, p.value);
          f.append("file", berkas);
          res = await fetchUnggah(s.url, { method: "POST", body: f, credentials: "omit", referrerPolicy: "no-referrer", signal: sinyal });
        } else {
          const h = new Headers();
          for (const p of s.headers) h.set(p.name, p.value);
          res = await fetchUnggah(s.url, { method: "PUT", body: berkas, headers: h, credentials: "omit", referrerPolicy: "no-referrer", signal: sinyal });
        }
        // Object storage menolak (slot kedaluwarsa, ukuran tidak cocok): tidak ada yang tercatat. Status 0, bukan
        // status object storage: 401/403 dari storage tidak boleh dibaca sebagai sesi berakhir atau akses ditolak.
        if (!res.ok) return { ok: false, status: 0, kode: null, pesan: "Berkas gagal diunggah. Pilih berkas lalu unggah lagi." };
      } catch {
        return gagalTerhubung();
      }
      return kerjakan(
        c.POST("/v1/corporate/documents", { body: { document_kind: jenis, upload_slot_id: slot.data.upload_slot_id } }),
        ringkasan,
      );
    },
    urlUnduh: (jenis) =>
      kerjakan(c.POST("/v1/corporate/document-download-urls", { body: { document_kind: jenis } }), (d) => d as UrlUnduh),
    ajukanUlang: () => kerjakan(c.POST("/v1/corporate/verification-resubmissions"), ringkasan),
    koreksiDanAjukanUlang: async (badan) => {
      let r: Respons;
      try {
        r = await c.POST("/v1/corporate/company-corrections", { body: badan });
      } catch {
        return { ...gagalTerhubung(), isian: {} };
      }
      if (!r.response.ok) return galatKoreksi(r.response.status, r.error);
      const amplop = (typeof r.data === "object" && r.data !== null ? r.data : {}) as { data?: unknown };
      return { ok: true, data: ringkasan(amplop.data) };
    },
  };
}
