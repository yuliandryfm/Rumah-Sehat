import { describe, expect, test } from "vitest";
import { galatKoreksi } from "./klien.ts";
import { badanKoreksi, galatServer, isianAwal, periksaKoreksi } from "./koreksi.ts";

const P = {
  legal_name: "PT Contoh Sintetis",
  nib: "1234567890123",
  npwp: "0123456789012345",
  office_address: "Jl. Contoh No. 1",
  office_phone: "021-0000000",
  submitted_at: "2027-03-01T01:15:00Z",
};
const TIDAK = { nib: false, npwp: false };

describe("model koreksi data perusahaan (CR-036)", () => {
  test("isian awal tidak memuat NIB/NPWP tersimpan (Q23 = A)", () => {
    const i = isianAwal(P);
    expect(i).toEqual({ legal_name: P.legal_name, nib: "", npwp: "", office_address: P.office_address, office_phone: P.office_phone });
  });

  test("badan hanya berisi field yang berubah (di-trim) dan NIB/NPWP yang dipilih diubah", () => {
    const i = { ...isianAwal(P), legal_name: ` ${P.legal_name} `, office_phone: "021-1", nib: " 9900000000001 ", npwp: "x" };
    expect(badanKoreksi(i, TIDAK, P)).toEqual({ office_phone: "021-1" });
    expect(badanKoreksi(i, { nib: true, npwp: false }, P)).toEqual({ office_phone: "021-1", nib: "9900000000001" });
    expect(badanKoreksi(isianAwal(P), TIDAK, P)).toEqual({});
  });

  test("validasi sama dengan formulir daftar, hanya untuk field yang dikirim", () => {
    const i = { ...isianAwal(P), legal_name: "", nib: "12", npwp: "099999999999999" };
    expect(periksaKoreksi(i, TIDAK, P)).toEqual({ legal_name: "Nama badan usaha wajib diisi." });
    const g = periksaKoreksi(i, { nib: true, npwp: true }, P);
    expect(g.nib).toMatch(/Format NIB tidak sesuai/);
    expect(g.npwp).toMatch(/Tambahkan angka 0 di depan/);
    expect(periksaKoreksi({ ...isianAwal(P), nib: "9900000000001" }, { nib: true, npwp: false }, P)).toEqual({});
  });

  test("galat server: hanya field koreksi yang dikenal; pesan per field dari errors[]", () => {
    expect(galatServer({ nib: "a", $: "b", status: "c" })).toEqual({ nib: "a" });
    const g = galatKoreksi(422, { response_code: "4220001", errors: [{ field: "npwp", message: "m" }, { field: "npwp", message: "m2" }, { field: 1 }] });
    expect(g).toMatchObject({ ok: false, status: 422, kode: "4220001", isian: { npwp: "m" } });
    expect(galatKoreksi(429, { response_code: "4290001" }).pesan).toMatch(/Terlalu banyak percobaan pengajuan ulang/);
    expect(galatKoreksi(409, { response_code: "4090115" }).pesan).toMatch(/Dokumen belum lengkap/);
  });
});
