import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { galat } from "../../lib/hr/klien.ts";
import { isiBeranda } from "../../lib/beranda/model.ts";
import type { KlienVerifikasi, Muatan, Ringkasan } from "../../lib/verifikasi/klien.ts";
import * as halaman from "../../app/(terautentikasi)/page.tsx";
import { Beranda } from "./Beranda.tsx";

vi.mock("next/navigation", () => ({ usePathname: () => "/" }));

afterEach(() => {
  cleanup();
});

const DOK = (a: boolean, b: boolean, c: boolean): Ringkasan["documents"] => [
  { kind: "nib", uploaded: a, uploaded_at: a ? "2027-03-01T01:00:00Z" : null },
  { kind: "npwp", uploaded: b, uploaded_at: b ? "2027-03-01T01:00:00Z" : null },
  { kind: "authority_proof", uploaded: c, uploaded_at: c ? "2027-03-01T01:00:00Z" : null },
];
const ringkasan = (o: Partial<Ringkasan> = {}): Ringkasan => ({
  status: "pending_verification",
  submission_round: 1,
  complete: false,
  rejection_reason: null,
  company: {
    legal_name: "PT Contoh Sintetis",
    nib: "1234567890123",
    npwp: "0123456789012345",
    office_address: "Jl. Contoh No. 1, Jakarta",
    office_phone: "021-0000000",
    submitted_at: "2027-03-01T01:15:00Z",
  },
  documents: DOK(false, false, false),
  ...o,
});

const ok = <T,>(data: T) => Promise.resolve({ ok: true as const, data });
const gagal = (status: number, kode?: string) => Promise.resolve(galat(status, kode === undefined ? {} : { response_code: kode }));
const dokumen = (r: Ringkasan): Muatan => ({ jenis: "dokumen", ringkasan: r });

function klien(muat: KlienVerifikasi["muat"]): KlienVerifikasi {
  const tak = () => Promise.reject(new Error("tidak dipakai Beranda"));
  return { muat, unggah: tak, urlUnduh: tak, ajukanUlang: tak, koreksiDanAjukanUlang: tak };
}

const KASUS: readonly (readonly [string, Muatan, string, string, string, string])[] = [
  ["dokumen belum lengkap", dokumen(ringkasan({ documents: DOK(true, false, false) })), "Lengkapi dokumen perusahaan", "Unggah dokumen", "/verifikasi", "Menunggu verifikasi"],
  ["menunggu verifikasi (dokumen lengkap)", dokumen(ringkasan({ complete: true, documents: DOK(true, true, true) })), "Dokumen sedang diperiksa", "Lihat status pemeriksaan", "/verifikasi", "Menunggu verifikasi"],
  ["ditolak", dokumen(ringkasan({ status: "rejected", complete: true, rejection_reason: "authority_proof_insufficient", documents: DOK(true, true, true) })), "Pengajuan perlu diperbaiki", "Perbaiki dan ajukan ulang", "/verifikasi", "Ditolak"],
  ["aktif (4090112)", { jenis: "aktif" }, "Perusahaan Anda sudah aktif", "Undang rekan HR", "/kelola-hr", "Aktif"],
];

describe("Beranda: satu status dan SATU tombol langkah berikutnya", () => {
  test.each(KASUS)("%s", async (_n, muatan, judul, tombol, href, status) => {
    render(<Beranda csrfToken="t" klien={klien(() => ok(muatan))} />);
    expect(await screen.findByRole("heading", { level: 2, name: judul })).toBeDefined();
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Beranda");
    expect(screen.getByText(status)).toBeDefined();
    // Tepat satu tautan/tombol aksi di isi Beranda, menuju halaman yang sudah ada.
    const aksi = screen.getAllByRole("link");
    expect(aksi).toHaveLength(1);
    expect(aksi[0]?.textContent).toBe(tombol);
    expect(aksi[0]?.getAttribute("href")).toBe(href);
    expect(screen.queryAllByRole("button")).toHaveLength(0);
    // Bukan dasbor angka dan tanpa data kesehatan peserta (BR-143).
    expect(document.body.textContent).not.toMatch(/\d{2,}|hasil pemeriksaan|diagnosis/);
  });

  test("dokumen belum lengkap: hitungan dokumen terunggah dari data, bukan karangan", async () => {
    render(<Beranda csrfToken="t" klien={klien(() => ok(dokumen(ringkasan({ documents: DOK(true, false, true) }))))} />);
    expect(await screen.findByText("2 dari 3 dokumen sudah diunggah.")).toBeDefined();
  });

  test("ditolak: alasan dalam bahasa biasa, bukan kode", async () => {
    const r = ringkasan({ status: "rejected", complete: true, rejection_reason: "company_data_mismatch", documents: DOK(true, true, true) });
    render(<Beranda csrfToken="t" klien={klien(() => ok(dokumen(r)))} />);
    expect(await screen.findByText("Alasan: Data perusahaan tidak sesuai dengan dokumen.")).toBeDefined();
    expect(document.body.textContent).not.toContain("company_data_mismatch");
  });

  test("aktif: Pesanan, Peserta, Tagihan hanya disebut 'segera hadir', bukan tautan", async () => {
    render(<Beranda csrfToken="t" klien={klien(() => ok({ jenis: "aktif" }))} />);
    expect(await screen.findByText("Menu Pesanan, Peserta, dan Tagihan segera hadir.")).toBeDefined();
    expect(screen.queryByRole("link", { name: /Pesanan|Peserta|Tagihan/ })).toBeNull();
  });

  test("memuat: kerangka tampil, belum ada tombol aksi", () => {
    render(<Beranda csrfToken="t" klien={klien(() => new Promise(() => undefined))} />);
    expect(screen.queryByRole("link")).toBeNull();
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Beranda");
  });
});

describe("Beranda: galat", () => {
  test("401 4010002 → layar akun nonaktif, tanpa tombol aksi Beranda", async () => {
    render(<Beranda csrfToken="t" klien={klien(() => gagal(401, "4010002"))} />);
    expect(await screen.findByRole("heading", { level: 1, name: "Akun nonaktif" })).toBeDefined();
    expect(screen.queryByRole("link", { name: "Unggah dokumen" })).toBeNull();
  });

  test("401 lain (sesi berakhir) → dialihkan ke /auth/login", async () => {
    const arahkan = vi.fn();
    render(<Beranda csrfToken="t" klien={klien(() => gagal(401, "4010001"))} arahkan={arahkan} />);
    await waitFor(() => {
      expect(arahkan).toHaveBeenCalledWith("/auth/login");
    });
  });

  test("503 → pesan bahasa biasa dengan 'Coba lagi', tanpa kode teknis dan tanpa tombol langkah", async () => {
    const muat = vi.fn<KlienVerifikasi["muat"]>().mockImplementationOnce(() => gagal(503)).mockImplementation(() => ok({ jenis: "aktif" }));
    render(<Beranda csrfToken="t" klien={klien(muat)} />);
    expect(await screen.findByText("Beranda belum bisa dimuat")).toBeDefined();
    expect(screen.getByRole("alert").textContent).toContain("Layanan sedang tidak tersedia");
    expect(screen.queryByRole("link")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Coba lagi" }));
    expect(await screen.findByRole("heading", { level: 2, name: "Perusahaan Anda sudah aktif" })).toBeDefined();
  });
});

describe("Beranda: status tak dikenal dan galat yang bukan milik Beranda", () => {
  test("status tak dikenal (walau dokumen lengkap) → keadaan netral, bukan 'sedang diperiksa'", async () => {
    const r = { ...ringkasan({ complete: true, documents: DOK(true, true, true) }), status: "suspended" } as unknown as Ringkasan;
    render(<Beranda csrfToken="t" klien={klien(() => ok(dokumen(r)))} />);
    expect(await screen.findByRole("heading", { level: 2, name: "Status belum dapat ditampilkan" })).toBeDefined();
    expect(screen.getByRole("link", { name: "Lihat status" }).getAttribute("href")).toBe("/verifikasi");
    expect(document.body.textContent).not.toMatch(/sedang diperiksa|Unggah dokumen/);
  });

  test("status tak dikenal dan dokumen belum lengkap → tetap netral, bukan 'Lengkapi dokumen'", () => {
    const r = { ...ringkasan({ documents: DOK(true, false, false) }), status: "suspended" } as unknown as Ringkasan;
    expect(isiBeranda(dokumen(r)).kunci).toBe("netral");
  });

  test.each([
    ["403 (perusahaan belum aktif)", () => gagal(403)],
    ["409 4090115", () => gagal(409, "4090115")],
    ["409 4090001", () => gagal(409, "4090001")],
  ])("%s → pesan Beranda biasa dengan 'Coba lagi', bukan pesan halaman lain", async (_n, galatnya) => {
    render(<Beranda csrfToken="t" klien={klien(galatnya)} />);
    expect(await screen.findByText("Beranda belum bisa dimuat")).toBeDefined();
    expect(screen.getByRole("alert").textContent).toContain("Status perusahaan belum bisa ditampilkan");
    expect(screen.getByRole("button", { name: "Coba lagi" })).toBeDefined();
    expect(screen.queryByRole("link")).toBeNull();
    expect(document.body.textContent).not.toMatch(/menu ini belum dapat dipakai|belum aktif, sehingga|Dokumen belum lengkap|tidak dapat diubah lagi/i);
  });
});

describe("isiBeranda (model)", () => {
  test("tujuan tombol hanya halaman yang ada, dan label tiap keadaan berbeda", () => {
    const isi = KASUS.map(([, m]) => isiBeranda(m));
    expect(new Set(isi.map((i) => i.tindakan.href))).toEqual(new Set(["/verifikasi", "/kelola-hr"]));
    expect(new Set(isi.map((i) => i.kunci)).size).toBe(4);
  });
});

describe("halaman", () => {
  test("rute Beranda mengekspor halaman server (tanpa sesi → login lewat wajibSesi)", () => {
    expect(typeof halaman.default).toBe("function");
  });
});
