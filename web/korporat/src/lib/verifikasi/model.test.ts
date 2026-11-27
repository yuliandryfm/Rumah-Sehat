import { describe, expect, test } from "vitest";
import { buatCsp, originUnggahDariEnv } from "../../keamanan.ts";
import { galat } from "./klien.ts";
import { BATAS_FOTO, BATAS_PDF, DOKUMEN, formatWib, labelAlasan, periksaBerkas, termasking } from "./model.ts";

describe("pemeriksaan berkas di klien (NFR-SEC-08; server tetap penentu)", () => {
  test("JPG/PNG sampai 5 MiB dan PDF sampai 10 MiB boleh dicoba", () => {
    expect(periksaBerkas({ type: "image/jpeg", size: BATAS_FOTO })).toBeNull();
    expect(periksaBerkas({ type: "image/png", size: 1 })).toBeNull();
    expect(periksaBerkas({ type: "application/pdf", size: BATAS_PDF })).toBeNull();
  });

  test("negatif: batas per jenis, jenis lain, berkas kosong, tanpa berkas", () => {
    expect(periksaBerkas({ type: "image/jpeg", size: BATAS_FOTO + 1 })).toMatch(/Foto JPG atau PNG paling besar 5 MiB/);
    expect(periksaBerkas({ type: "application/pdf", size: BATAS_PDF + 1 })).toMatch(/PDF paling besar 10 MiB/);
    expect(periksaBerkas({ type: "image/webp", size: 10 })).toMatch(/Jenis berkas tidak didukung/);
    expect(periksaBerkas({ type: "", size: 10 })).toMatch(/Jenis berkas tidak didukung/);
    expect(periksaBerkas({ type: "image/png", size: 0 })).toMatch(/kosong/);
    expect(periksaBerkas(null)).toMatch(/Pilih berkas/);
  });
});

describe("tampilan data", () => {
  test("tiga dokumen wajib berurutan NIB, NPWP, bukti kewenangan (surat kuasa/penugasan direksi)", () => {
    expect(DOKUMEN.map((d) => d.jenis)).toEqual(["nib", "npwp", "authority_proof"]);
    expect(DOKUMEN[2]?.bantuan).toMatch(/surat kuasa atau surat penugasan dari direksi/i);
  });

  test("NIB/NPWP termasking: hanya empat digit terakhir", () => {
    expect(termasking("1234567890123")).toBe("************0123");
    expect(termasking("0123456789012345")).toBe("************2345");
  });

  test("alasan penolakan dari kode; kode tidak dikenal atau kosong → teks umum, bukan kode mentah", () => {
    expect(labelAlasan("authority_proof_insufficient")).toBe("Bukti kewenangan pendaftar tidak memadai");
    expect(labelAlasan("kode_baru")).toBe("Pengajuan belum memenuhi syarat verifikasi");
    expect(labelAlasan(null)).toBe("Pengajuan belum memenuhi syarat verifikasi");
  });

  test("waktu berlabel WIB tanpa bergantung zona peramban", () => {
    expect(formatWib("2027-03-01T01:15:00Z")).toMatch(/1 Mar 2027.*08\.15 WIB$/);
    expect(formatWib("bukan tanggal")).toBe("-");
    expect(formatWib(null)).toBe("-");
  });
});

describe("pesan galat per kode (bukan isi respons mentah)", () => {
  test("4090001 di layar verifikasi berarti 'sudah menunggu', berbeda dari Kelola HR", () => {
    expect(galat(409, { response_code: "4090001", response_description: "x" }).pesan).toMatch(/sudah menunggu verifikasi/);
    expect(galat(409, { response_code: "4090115" }).pesan).toMatch(/Dokumen belum lengkap/);
    expect(galat(409, { response_code: "4090112" }).pesan).toMatch(/sudah Aktif/);
    expect(galat(415, {}).pesan).toMatch(/PDF bertanda tangan digital/);
    expect(galat(500, { response_description: "<script>bocor</script>" }).pesan).not.toContain("bocor");
  });
});

describe("CSP unggah dokumen (F1-T103, ADR-018)", () => {
  test("satu origin object storage masuk connect-src; tanpa konfigurasi tetap 'self' saja", () => {
    expect(buatCsp("n", false, "https://unggah.contoh.test")).toContain("connect-src 'self' https://unggah.contoh.test;");
    expect(buatCsp("n")).toContain("connect-src 'self';");
    expect(buatCsp("n", false, "https://unggah.contoh.test")).toContain("img-src 'self';");
  });

  test("negatif: env bukan origin https murni diabaikan (gagal tertutup)", () => {
    expect(originUnggahDariEnv("https://unggah.contoh.test")).toBe("https://unggah.contoh.test");
    expect(originUnggahDariEnv("https://unggah.contoh.test/")).toBe("https://unggah.contoh.test");
    for (const nilai of [undefined, "", "http://unggah.contoh.test", "https://unggah.contoh.test/bucket", "https://u:p@unggah.contoh.test", "https://unggah.contoh.test/?a=1", "*", "bukan url"]) {
      expect(originUnggahDariEnv(nilai)).toBeNull();
    }
    expect(originUnggahDariEnv("http://127.0.0.1:18333", true)).toBe("http://127.0.0.1:18333");
  });
});
