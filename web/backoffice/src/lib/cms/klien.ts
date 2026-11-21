import { createBackofficeClient, type BackofficeComponents } from "@rs/api-client";
import type { GalatIsian, HalamanSitus, Kanal, Konten, PermintaanKonten, StatusTampil } from "./model.ts";

type S = BackofficeComponents["schemas"];

export interface Gagal {
  readonly ok: false;
  readonly status: number;
  readonly pesan: string;
  readonly isian: readonly GalatIsian[];
  readonly kode: string | null;
}
export type Hasil<T> = { readonly ok: true; readonly data: T } | Gagal;

/** Awalan BFF (`lib/server/bff.ts`); path API Rust: `/backoffice/v1/...`. */
export const BASIS_BFF = "/bff";

const PESAN_STATUS: Record<number, string> = {
  401: "Sesi Anda berakhir. Masuk lagi untuk melanjutkan.",
  403: "Akun Anda tidak memiliki sub-peran Konten untuk tindakan ini.",
  404: "Konten tidak ditemukan.",
  409: "Konten dalam status ini tidak dapat diubah. Buat versi baru.",
  413: "Ukuran berkas melebihi batas 5 MiB.",
  415: "Hanya gambar JPG atau PNG yang diizinkan.",
  429: "Terlalu banyak permintaan. Coba lagi sebentar lagi.",
  503: "Layanan sedang tidak tersedia. Coba lagi nanti.",
};

/** Galat dari envelope RFC 9457 (ADR-023) atau galat BFF; isinya tidak dipercaya dan hanya dibaca sebagai teks. */
export function galat(status: number, badan: unknown): Gagal {
  const b = (typeof badan === "object" && badan !== null ? badan : {}) as Partial<S["ErrorEnvelope"]>;
  const isian = Array.isArray(b.errors)
    ? b.errors.filter((e: unknown): e is GalatIsian => {
        const x = e as Partial<GalatIsian> | null;
        return typeof x?.field === "string" && typeof x.message === "string";
      })
    : [];
  const pesan =
    PESAN_STATUS[status] ??
    (typeof b.response_description === "string" && b.response_description !== ""
      ? b.response_description
      : "Permintaan gagal. Coba lagi.");
  return { ok: false, status, pesan, isian, kode: typeof b.response_code === "string" ? b.response_code : null };
}

export interface Halaman {
  readonly items: readonly Konten[];
  readonly hasMore: boolean;
  readonly nextCursor: string | null;
}

export interface KlienCms {
  daftar(q: { status?: StatusTampil; channel?: Kanal; site_page?: HalamanSitus; cursor?: string }): Promise<Hasil<Halaman>>;
  ambil(id: string): Promise<Hasil<Konten>>;
  buat(isi: PermintaanKonten): Promise<Hasil<Konten>>;
  ubah(id: string, isi: PermintaanKonten): Promise<Hasil<Konten>>;
  jadwalkan(id: string, mulai: string | null, selesai: string | null): Promise<Hasil<Konten>>;
  arsipkan(id: string): Promise<Hasil<Konten>>;
  hapus(id: string, alasan: string): Promise<Hasil<{ content_id: string }>>;
  pulihkan(id: string): Promise<Hasil<Konten>>;
  unggahGambar(id: string, berkas: File, teksAlt: string): Promise<Hasil<Konten>>;
}

interface Respons {
  readonly data?: unknown;
  readonly error?: unknown;
  readonly response: Response;
}

/**
 * Klien CMS dari browser lewat BFF. Token CSRF sesi dikirim di `X-CSRF-Token` (token OIDC tidak pernah
 * ada di browser). `opsi` hanya untuk test (di browser `basis` relatif `/bff`).
 */
export interface OpsiKlienCms {
  readonly basis?: string;
  readonly fetch?: typeof fetch;
  readonly fetchUnggah?: typeof fetch;
}

export function buatKlienCms(csrfToken: string, opsi: OpsiKlienCms = {}): KlienCms {
  const fetchUnggah: typeof fetch = opsi.fetchUnggah ?? ((...a) => fetch(...a));
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
  const konten = (d: unknown) => d as Konten;
  const path = (id: string) => ({ params: { path: { content_id: id } } });
  return {
    daftar: (q) =>
      kerjakan(c.GET("/backoffice/v1/contents", { params: { query: { ...q, limit: 50 } } }), (d, r) => {
        const meta = ((r.data as { meta?: S["MetaSchema"] }).meta ?? { has_more: false });
        return { items: (d ?? []) as Konten[], hasMore: meta.has_more, nextCursor: meta.next_cursor ?? null };
      }),
    ambil: (id) => kerjakan(c.GET("/backoffice/v1/contents/{content_id}", path(id)), konten),
    buat: (body) => kerjakan(c.POST("/backoffice/v1/contents", { body }), konten),
    ubah: (id, body) => kerjakan(c.PUT("/backoffice/v1/contents/{content_id}", { ...path(id), body }), konten),
    jadwalkan: (id, mulai, selesai) =>
      kerjakan(
        c.POST("/backoffice/v1/contents/{content_id}/schedule", {
          ...path(id),
          body: { publish_start_at: mulai, publish_end_at: selesai },
        }),
        konten,
      ),
    arsipkan: (id) => kerjakan(c.POST("/backoffice/v1/contents/{content_id}/archive", path(id)), konten),
    hapus: (id, alasan) =>
      kerjakan(
        c.POST("/backoffice/v1/contents/{content_id}/delete", { ...path(id), body: { delete_reason: alasan } }),
        (d) => d as { content_id: string },
      ),
    pulihkan: (id) => kerjakan(c.POST("/backoffice/v1/contents/{content_id}/restore", path(id)), konten),
    unggahGambar: async (id, berkas, teksAlt) => {
      const slot = await kerjakan(
        c.POST("/backoffice/v1/content-images/upload-slots", { body: { content_type: berkas.type, size_bytes: berkas.size } }),
        (d) => d as S["UploadSlotDto"],
      );
      if (!slot.ok) return slot;
      try {
        // URL bertanda tangan ke object storage (ADR-018): tanpa cookie dan tanpa CSRF. Isi berkas diperiksa backend saat dipasang.
        const s = slot.data;
        let res: Response;
        if (s.method === "post") {
          const f = new FormData();
          for (const p of s.fields) f.append(p.name, p.value);
          f.append("file", berkas);
          res = await fetchUnggah(s.url, { method: "POST", body: f, credentials: "omit", referrerPolicy: "no-referrer" });
        } else {
          const h = new Headers();
          for (const p of s.headers) h.set(p.name, p.value);
          res = await fetchUnggah(s.url, { method: "PUT", body: berkas, headers: h, credentials: "omit", referrerPolicy: "no-referrer" });
        }
        if (!res.ok) return galat(503, null);
      } catch {
        return galat(503, null);
      }
      return kerjakan(
        c.PUT("/backoffice/v1/contents/{content_id}/image", {
          ...path(id),
          body: { upload_slot_id: slot.data.upload_slot_id, image_alt_text: teksAlt },
        }),
        konten,
      );
    },
  };
}
