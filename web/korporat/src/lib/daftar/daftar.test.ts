import { describe, expect, test } from "vitest";
import { buatKlienDaftar, galatDaftar } from "./klien.ts";
import { ISIAN_KOSONG, badanPermintaan, langkahDari, periksaLangkah, type Isian } from "./model.ts";

const ISIAN: Isian = {
  legal_name: " PT Contoh Sintetis ",
  nib: "1234567890123",
  npwp: "0123456789012345",
  office_address: "Jl. Contoh No. 1, Jakarta",
  office_phone: "021-0000000",
  hr_full_name: "Nama HR Sintetis",
  hr_job_title: "Manajer SDM",
  hr_email: "hr@perusahaan.example",
};
const VERSI = "01a2008d-7400-7000-8000-000000000b01";
const BUTIR = "01a2008d-7400-7000-8000-000000000b02";

describe("validasi klien pendaftaran (US-1901 AC3; Q6; server tetap penentu)", () => {
  test("isian benar lolos per langkah", () => {
    expect(periksaLangkah(0, ISIAN)).toEqual({});
    expect(periksaLangkah(1, ISIAN)).toEqual({});
  });

  test("negatif: wajib per field, NIB bukan 13 angka, NPWP bukan 16 angka, email salah", () => {
    expect(Object.keys(periksaLangkah(0, ISIAN_KOSONG))).toEqual(["legal_name", "nib", "npwp", "office_address", "office_phone"]);
    expect(periksaLangkah(0, { ...ISIAN, nib: "123456789012" }).nib).toMatch(/NIB berisi 13 angka/);
    expect(periksaLangkah(0, { ...ISIAN, nib: "12345678901a3" }).nib).toMatch(/Format NIB tidak sesuai/);
    expect(periksaLangkah(0, { ...ISIAN, npwp: "01.234.567.8-901.000" }).npwp).toMatch(/16 angka tanpa titik/);
    expect(periksaLangkah(1, { ...ISIAN, hr_email: "bukan-email" }).hr_email).toMatch(/Format email/);
  });

  test("NPWP lama 15 angka → petunjuk tambahkan 0 di depan; tidak ditambahkan diam-diam (KEPUTUSAN-325)", () => {
    expect(periksaLangkah(0, { ...ISIAN, npwp: "123456789012345" }).npwp).toMatch(/Tambahkan angka 0 di depan/);
    expect(badanPermintaan({ ...ISIAN, npwp: "123456789012345" }, VERSI, []).npwp).toBe("123456789012345");
  });

  test("badan permintaan: nilai di-trim, versi teks dan butir dikirim, tanpa field lain (tanpa password)", () => {
    const b = badanPermintaan(ISIAN, VERSI, [BUTIR]);
    expect(b.legal_name).toBe("PT Contoh Sintetis");
    expect(Object.keys(b).sort()).toEqual(
      ["consent_item_ids", "consent_text_version_id", "hr_email", "hr_full_name", "hr_job_title", "legal_name", "nib", "npwp", "office_address", "office_phone"].sort(),
    );
    expect(b.consent_item_ids).toEqual([BUTIR]);
  });

  test("galat 422 server dipetakan ke langkah field", () => {
    expect(langkahDari("npwp")).toBe(0);
    expect(langkahDari("hr_email")).toBe(1);
    expect(langkahDari("consent_item_ids")).toBe(2);
  });
});

describe("galat respons (tanpa isi mentah)", () => {
  test("409 4090106 → teks berubah; 422 → pesan per isian dari errors[]; 429; 503", () => {
    expect(galatDaftar(409, { response_code: "4090106" }).pesan).toMatch(/Teks persetujuan telah berubah/);
    const g = galatDaftar(422, { response_code: "4220001", errors: [{ field: "npwp", code: "invalid_format", message: "NPWP harus 16 angka" }] });
    expect(g.isian).toEqual({ npwp: "NPWP harus 16 angka" });
    expect(galatDaftar(429, {}).pesan).toMatch(/Terlalu banyak permintaan/);
    expect(galatDaftar(503, {}).pesan).toMatch(/tidak dapat diproses/);
    expect(galatDaftar(500, { response_description: "<b>rincian internal</b>" }).pesan).not.toContain("rincian");
  });
});

describe("klien pendaftaran publik (Q30 = C): langsung ke API publik di asal yang sama, bukan BFF", () => {
  const tangkap = () => {
    const permintaan: Request[] = [];
    const f = ((r: Request) => {
      permintaan.push(r);
      const teks = r.url.endsWith("/consent-text");
      return Promise.resolve(
        new Response(
          JSON.stringify({
            response_code: teks ? "2000000" : "2020000",
            response_description: "x",
            data: teks ? { consent_text_version_id: VERSI } : { status: "processing", message: "Pesan seragam server" },
          }),
          { status: teks ? 200 : 202, headers: { "content-type": "application/json" } },
        ),
      );
    }) as typeof fetch;
    return { permintaan, f };
  };

  test("GET teks persetujuan dan POST pendaftaran: path publik, tanpa cookie, tanpa header CSRF, tanpa /bff", async () => {
    const { permintaan, f } = tangkap();
    const k = buatKlienDaftar({ basis: "https://member.contoh.test", fetch: f });
    const t = await k.teks();
    expect(t.ok && t.data.consent_text_version_id).toBe(VERSI);
    const h = await k.kirim(ISIAN, VERSI, [BUTIR]);
    expect(h).toEqual({ ok: true, data: { status: "processing", message: "Pesan seragam server" } });
    expect(permintaan.map((r) => `${r.method} ${r.url}`)).toEqual([
      "GET https://member.contoh.test/v1/public/corporate-registrations/consent-text",
      "POST https://member.contoh.test/v1/public/corporate-registrations",
    ]);
    for (const r of permintaan) {
      expect(r.credentials).toBe("omit");
      expect(r.headers.get("x-csrf-token")).toBeNull();
      expect(r.headers.get("cookie")).toBeNull();
      expect(r.url).not.toContain("/bff");
    }
    expect(JSON.parse((await permintaan[1]?.text()) ?? "{}")).toMatchObject({ consent_text_version_id: VERSI, consent_item_ids: [BUTIR], legal_name: "PT Contoh Sintetis" });
  });

  test("negatif: gagal jaringan atau batas waktu → status 0, pesan belum terkirim", async () => {
    const tolak = (() => Promise.reject(new TypeError("fetch failed"))) as typeof fetch;
    const h = await buatKlienDaftar({ basis: "https://member.contoh.test", fetch: tolak }).kirim(ISIAN, VERSI, [BUTIR]);
    expect(h).toMatchObject({ ok: false, status: 0 });
    expect(!h.ok && h.pesan).toMatch(/Gagal terhubung. Pendaftaran belum terkirim/);
  });
});
