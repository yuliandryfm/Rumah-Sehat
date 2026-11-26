import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { ITEM_NAV, menuUntuk } from "../../lib/navigasi.ts";
import { buatKlienPerusahaan, galatPerusahaan, type KlienPerusahaan } from "../../lib/perusahaan/klien.ts";
import {
  nibTermasking,
  sumber,
  type DetailPerusahaan,
  type ItemAntreanPerusahaan,
} from "../../lib/perusahaan/model.ts";
import * as halaman from "../../app/(terautentikasi)/verifikasi-perusahaan/page.tsx";
import { PanelPerusahaan } from "./PanelPerusahaan.tsx";

const NIB = "9120000000099";
const NPWP = "012345678901234";
const ID = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
const VERSI = "a".repeat(64);
const URL_DOK = "https://neo.example.test/dok/abc?X-Sig=rahasia";

const item = (o: Partial<ItemAntreanPerusahaan> = {}): ItemAntreanPerusahaan => ({
  corporate_client_id: ID,
  legal_name: "PT Sintetis Sejahtera",
  nib: NIB,
  origin: "self_registered",
  submission_round: 1,
  submitted_at: "2027-03-01T01:01:00Z",
  ...o,
});
const detail = (o: Partial<DetailPerusahaan> = {}): DetailPerusahaan => ({
  corporate_client_id: ID,
  legal_name: "PT Sintetis Sejahtera",
  nib: NIB,
  npwp: NPWP,
  office_address: "Jl. Contoh 1, Jakarta",
  office_phone: "02100000000",
  status: "pending_verification",
  origin: "self_registered",
  submission_round: 1,
  submitted_at: "2027-03-01T01:01:00Z",
  documents: ["nib", "npwp", "authority_proof"],
  documents_version: VERSI,
  can_decide: true,
  ...o,
});

const ok = <T,>(data: T) => Promise.resolve({ ok: true as const, data });
const gagal = (status: number, kode?: string) =>
  Promise.resolve(galatPerusahaan(status, kode === undefined ? {} : { response_code: kode }));

function klienUji(o: Partial<KlienPerusahaan> = {}): KlienPerusahaan {
  return {
    antrean: () => ok({ items: [item()], hasMore: false, nextCursor: null }),
    alasan: () => ok([{ code: "document_invalid", label: "Dokumen tidak valid" }]),
    detail: () => ok(detail()),
    dokumenUrl: () => ok({ url: URL_DOK, expires_at: new Date(Date.now() + 60_000).toISOString(), mime_type: "image/jpeg", size_bytes: 10 }),
    setujui: () => ok({ corporate_client_id: ID, status: "active" as const, submission_round: 1 }),
    tolak: () => ok({ corporate_client_id: ID, status: "rejected" as const, submission_round: 1 }),
    buat: () => ok({ corporate_client_id: ID, status: "active", corporate_member_id: ID, hr_account_id: ID, hr_invitation: "processing" }),
    ...o,
  };
}

const konsol = { log: vi.spyOn(console, "log"), warn: vi.spyOn(console, "warn"), error: vi.spyOn(console, "error"), info: vi.spyOn(console, "info") };
beforeEach(() => {
  for (const s of Object.values(konsol)) s.mockImplementation(() => undefined);
});
afterEach(() => {
  cleanup();
  for (const s of Object.values(konsol)) s.mockReset();
  vi.useRealTimers();
});

async function bukaPanel(k: KlienPerusahaan) {
  render(<PanelPerusahaan csrfToken="csrf" klien={k} />);
  fireEvent.click(await screen.findByRole("button", { name: /Buka\s*pengajuan/ }));
  await screen.findByTestId("npwp-lengkap");
}

describe("antrean", () => {
  test("NIB termasking, jam WIB, tanpa NIB/NPWP lengkap sebelum dibuka; daftar perusahaan ditandai belum tersedia", async () => {
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji()} />);
    expect(await screen.findByText("************0099")).toBeTruthy();
    expect(screen.getByText(/1 Mar 2027, 08.01 WIB/)).toBeTruthy();
    expect(document.body.textContent).not.toContain(NIB);
    expect(document.body.textContent).not.toContain(NPWP);
    expect(screen.getByTestId("belum-tersedia").textContent).toContain("belum tersedia");
  });

  test("antrean kosong dan muat lebih banyak", async () => {
    const antrean = vi
      .fn()
      .mockImplementationOnce(() => ok({ items: [item()], hasMore: true, nextCursor: "c1" }))
      .mockImplementationOnce(() => ok({ items: [item({ corporate_client_id: "x2", legal_name: "CV Dua" })], hasMore: false, nextCursor: null }));
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ antrean })} />);
    fireEvent.click(await screen.findByRole("button", { name: "Muat lebih banyak" }));
    expect(await screen.findByText("CV Dua")).toBeTruthy();
    expect(antrean).toHaveBeenLastCalledWith("c1");
    cleanup();
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ antrean: () => ok({ items: [], hasMore: false, nextCursor: null }) })} />);
    expect(await screen.findByText("Antrean kosong")).toBeTruthy();
  });

  test("pengajuan ulang berlabel pada kolom sumber", () => {
    expect(sumber(item({ submission_round: 2 }))).toBe("Pengajuan ulang");
    expect(sumber(item({ origin: "created_by_admin" }))).toBe("Dibuat admin");
    expect(nibTermasking(NIB)).toBe("************0099");
  });
});

describe("panel detail dan dokumen", () => {
  test("NIB/NPWP lengkap hanya setelah dibuka; dokumen hanya setelah diminta, tanpa referrer", async () => {
    const dokumenUrl = vi.fn(() => klienUji().dokumenUrl(ID, "nib"));
    await bukaPanel(klienUji({ dokumenUrl }));
    expect(screen.getByTestId("nib-lengkap").textContent).toContain(NIB);
    expect(screen.getByTestId("npwp-lengkap").textContent).toBe(NPWP);
    expect(document.querySelector("img")).toBeNull();
    expect(dokumenUrl).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: /Buka\s*Dokumen NIB/ }));
    await waitFor(() => { expect(document.querySelector("img")).not.toBeNull(); });
    expect(document.querySelector("img")?.getAttribute("referrerpolicy")).toBe("no-referrer");
    expect(document.querySelector("img")?.getAttribute("src")).toBe(URL_DOK);
    expect(dokumenUrl).toHaveBeenCalledWith(ID, "nib");
  });

  test("dokumen non-gambar hanya tautan tab baru tanpa referrer", async () => {
    await bukaPanel(
      klienUji({ dokumenUrl: () => ok({ url: URL_DOK, expires_at: new Date(Date.now() + 60_000).toISOString(), mime_type: "application/pdf", size_bytes: 10 }) }),
    );
    fireEvent.click(screen.getByRole("button", { name: /Buka\s*Bukti kewenangan/ }));
    const a = await screen.findByRole("link", { name: /tab baru/ });
    expect(a.getAttribute("rel")).toContain("noopener");
    expect(a.getAttribute("referrerpolicy")).toBe("no-referrer");
  });

  test("menutup panel membuang NIB/NPWP lengkap dan dokumen dari DOM; tanpa penyimpanan peramban", async () => {
    await bukaPanel(klienUji());
    fireEvent.click(screen.getByRole("button", { name: /Buka\s*Dokumen NIB/ }));
    await waitFor(() => { expect(document.querySelector("img")).not.toBeNull(); });
    fireEvent.click(screen.getByRole("button", { name: "Tutup" }));
    expect(document.querySelector("img")).toBeNull();
    expect(document.body.textContent).not.toContain(NPWP);
    expect(localStorage.length).toBe(0);
    expect(sessionStorage.length).toBe(0);
  });

  test("dokumen dilepas saat URL kedaluwarsa", async () => {
    await bukaPanel(klienUji({ dokumenUrl: () => ok({ url: URL_DOK, expires_at: new Date(Date.now() + 400).toISOString(), mime_type: "image/png", size_bytes: 1 }) }));
    fireEvent.click(screen.getByRole("button", { name: /Buka\s*Dokumen NIB/ }));
    await waitFor(() => { expect(document.querySelector("img")).not.toBeNull(); });
    await waitFor(() => { expect(document.querySelector("img")).toBeNull(); }, { timeout: 3000 });
  });

  test("pengajuan yang sudah diputuskan: tanpa tombol keputusan", async () => {
    await bukaPanel(klienUji({ detail: () => ok(detail({ can_decide: false, status: "active" })) }));
    expect(screen.queryByRole("button", { name: "Setujui" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Tolak" })).toBeNull();
  });
});

describe("keputusan", () => {
  test("negatif: 403 4030101 (CSRF/Origin ditolak BFF) saat setujui → pesan muat ulang, bukan tuduhan tanpa sub-peran", async () => {
    await bukaPanel(klienUji({ setujui: () => gagal(403, "4030101") }));
    fireEvent.click(screen.getByRole("button", { name: "Setujui" }));
    expect(await screen.findByText(/Permintaan tidak dapat diverifikasi/)).toBeTruthy();
    expect(screen.queryByText(/tidak memiliki sub-peran Verifikator/)).toBeNull();
    expect(screen.queryByText("Tidak ada akses")).toBeNull();
  });

  test("negatif: 403 4030101 saat memuat antrean → bukan layar Tidak ada akses", async () => {
    render(<PanelPerusahaan csrfToken="t" klien={klienUji({ antrean: () => gagal(403, "4030101") })} />);
    expect(await screen.findByText(/Permintaan tidak dapat diverifikasi/)).toBeTruthy();
    expect(screen.queryByText("Tidak ada akses")).toBeNull();
  });

  test("setujui memanggil API dan menyegarkan antrean", async () => {
    const setujui = vi.fn(() => ok({ corporate_client_id: ID, status: "active" as const, submission_round: 1 }));
    const antrean = vi.fn(() => ok({ items: [item()], hasMore: false, nextCursor: null }));
    await bukaPanel(klienUji({ setujui, antrean }));
    fireEvent.click(screen.getByRole("button", { name: "Setujui" }));
    expect(await screen.findByText(/disetujui dan berstatus Aktif/)).toBeTruthy();
    expect(setujui).toHaveBeenCalledWith(ID, VERSI);
    expect(antrean).toHaveBeenCalledTimes(2);
    expect(screen.queryByTestId("npwp-lengkap")).toBeNull();
  });

  test("tolak tanpa alasan ditolak di klien; dengan alasan terkirim kode", async () => {
    const tolak = vi.fn(() => ok({ corporate_client_id: ID, status: "rejected" as const, submission_round: 1 }));
    await bukaPanel(klienUji({ tolak }));
    fireEvent.click(screen.getByRole("button", { name: "Tolak" }));
    expect(await screen.findByText("Pilih alasan untuk menolak.")).toBeTruthy();
    expect(tolak).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText("Alasan penolakan"), { target: { value: "document_invalid" } });
    fireEvent.click(screen.getByRole("button", { name: "Tolak" }));
    expect(await screen.findByText(/Pengajuan ditolak/)).toBeTruthy();
    expect(tolak).toHaveBeenCalledWith(ID, "document_invalid", VERSI);
  });

  test("4090114 dokumen berubah: pesan dan detail dimuat ulang", async () => {
    const detailFn = vi.fn(() => ok(detail()));
    await bukaPanel(klienUji({ detail: detailFn, setujui: () => gagal(409, "4090114") }));
    fireEvent.click(screen.getByRole("button", { name: "Setujui" }));
    expect(await screen.findByText(/Dokumen perusahaan berubah/)).toBeTruthy();
    expect(detailFn).toHaveBeenCalledTimes(2);
  });

  test("4090001 keputusan pertama berlaku: pesan, tombol hilang, antrean disegarkan", async () => {
    const antrean = vi.fn(() => ok({ items: [item()], hasMore: false, nextCursor: null }));
    await bukaPanel(klienUji({ antrean, setujui: () => gagal(409, "4090001") }));
    fireEvent.click(screen.getByRole("button", { name: "Setujui" }));
    expect(await screen.findByText(/sudah diputuskan Verifikator lain/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Setujui" })).toBeNull();
    expect(antrean).toHaveBeenCalledTimes(2);
  });
});

describe("negatif", () => {
  test("403 non-Verifikator: layar tanpa akses, tanpa data", async () => {
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ antrean: () => gagal(403), alasan: () => gagal(403) })} />);
    expect(await screen.findByText("Tidak ada akses")).toBeTruthy();
    expect(screen.queryByRole("table")).toBeNull();
    expect(document.body.textContent).not.toContain("PT Sintetis");
  });

  test("503 pada antrean dan galat detail tampil sebagai pesan", async () => {
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ antrean: () => gagal(503) })} />);
    expect(await screen.findByText("Antrean gagal dimuat")).toBeTruthy();
    cleanup();
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ detail: () => gagal(404) })} />);
    fireEvent.click(await screen.findByRole("button", { name: /Buka\s*pengajuan/ }));
    expect(await screen.findByText(/tidak ditemukan/)).toBeTruthy();
  });

  test("NIB/NPWP tidak bocor ke console pada alur sukses maupun galat", async () => {
    await bukaPanel(klienUji({ setujui: () => gagal(409, "4090001") }));
    fireEvent.click(screen.getByRole("button", { name: "Setujui" }));
    await screen.findAllByText(/sudah diputuskan/);
    const semua = Object.values(konsol).flatMap((s) => s.mock.calls.flat().map(String)).join(" ");
    expect(semua).not.toContain(NIB);
    expect(semua).not.toContain(NPWP);
  });
});

describe("klien, navigasi, halaman", () => {
  test("galat memilih teks dari kode, bukan dari isi respons", () => {
    const g = galatPerusahaan(409, { response_code: "4090001", response_description: "<script>x</script>" });
    expect(g.pesan).not.toContain("script");
    expect(g.pesan).toContain("Keputusan pertama");
  });

  test("mutasi membawa X-CSRF-Token, jalur BFF benar, badan setujui/tolak memuat versi dokumen", async () => {
    const dipanggil: { url: string; method: string; csrf: string | null; body: string }[] = [];
    const fetchUji = (async (r: Request) => {
      dipanggil.push({ url: new URL(r.url).pathname, method: r.method, csrf: r.headers.get("x-csrf-token"), body: await r.text() });
      return new Response(JSON.stringify({ response_code: "2000000", response_description: "Berhasil", data: { url: "u" } }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    }) as unknown as typeof fetch;
    const k = buatKlienPerusahaan("tok", { basis: "http://bff.test/bff", fetch: fetchUji });
    await k.setujui(ID, VERSI);
    await k.tolak(ID, "document_invalid", VERSI);
    await k.dokumenUrl(ID, "npwp");
    await k.detail(ID);
    const dasar = `/bff/backoffice/v1/corporate-verifications/${ID}`;
    expect(dipanggil.map((d) => `${d.method} ${d.url}`)).toEqual([
      `POST ${dasar}/approve`,
      `POST ${dasar}/reject`,
      `POST ${dasar}/documents/npwp/download-url`,
      `GET ${dasar}`,
    ]);
    expect(dipanggil.every((d) => d.csrf === "tok")).toBe(true);
    expect(JSON.parse(dipanggil[1]?.body ?? "")).toEqual({ reason_code: "document_invalid", documents_version: VERSI });
    expect(JSON.parse(dipanggil[0]?.body ?? "")).toEqual({ documents_version: VERSI });
  });

  test("galat jaringan menjadi 503 terkendali", async () => {
    const k = buatKlienPerusahaan("t", { basis: "http://bff.test/bff", fetch: (() => Promise.reject(new Error("x"))) as unknown as typeof fetch });
    expect(await k.antrean()).toMatchObject({ ok: false, status: 503 });
  });

  test("menu verifikasi perusahaan hanya untuk Verifikator; halaman mengekspor komponen", () => {
    expect(ITEM_NAV.some((i) => i.href === "/verifikasi-perusahaan")).toBe(true);
    expect(menuUntuk(["Verifikator"]).some((i) => i.href === "/verifikasi-perusahaan")).toBe(true);
    expect(menuUntuk(["Konten"]).some((i) => i.href === "/verifikasi-perusahaan")).toBe(false);
    expect(typeof halaman.default).toBe("function");
  });
});

describe("Buat perusahaan (US-1903)", () => {
  const ISIAN = {
    "Nama badan usaha": "PT Contoh Sintetis",
    NIB: "9120000000123",
    NPWP: "012345678901299",
    "Alamat kantor": "Jl. Contoh No. 1, Jakarta",
    "Telepon kantor": "021-0000000",
    "Email HR pertama": "hr.pertama@perusahaan.example",
  };
  const deskripsi = (el: HTMLElement) =>
    (el.getAttribute("aria-describedby") ?? "")
      .split(" ")
      .map((i) => document.getElementById(i)?.textContent ?? "")
      .join(" ");

  async function isiForm(isian: Record<string, string> = ISIAN) {
    fireEvent.click(await screen.findByRole("button", { name: "Buat perusahaan" }));
    for (const [label, nilai] of Object.entries(isian)) {
      fireEvent.change(await screen.findByLabelText(label), { target: { value: nilai } });
    }
  }
  const kirim = () => {
    const submit = screen.getAllByRole("button", { name: "Buat perusahaan" }).find((b) => b.getAttribute("type") === "submit");
    if (submit === undefined) throw new Error("tombol submit tidak ada");
    fireEvent.click(submit);
  };

  test("sukses 201: kirim isian, pesan Aktif + undangan diproses, formulir tertutup, antrean dimuat ulang, tanpa data sensitif di layar", async () => {
    const buat = vi.fn(() => ok({ corporate_client_id: ID, status: "active", corporate_member_id: ID, hr_account_id: ID, hr_invitation: "processing" }));
    const antrean = vi.fn(() => ok({ items: [item()], hasMore: false, nextCursor: null }));
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ buat, antrean })} />);
    await isiForm();
    kirim();
    expect(await screen.findByText(/Perusahaan dibuat dan berstatus Aktif/)).toBeTruthy();
    expect(buat).toHaveBeenCalledWith(expect.objectContaining({ nib: ISIAN.NIB, hr_email: ISIAN["Email HR pertama"] }));
    expect(antrean).toHaveBeenCalledTimes(2);
    expect(screen.queryByLabelText("NIB")).toBeNull();
    expect(document.body.textContent).not.toContain(ISIAN.NIB);
    expect(document.body.textContent).not.toContain(ISIAN["Email HR pertama"]);
  });

  test("catatan jujur: unggah dokumen menyusul (F1-T102), tidak ada kontrol unggah palsu", async () => {
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji()} />);
    await isiForm({});
    expect(screen.getByTestId("dokumen-menyusul").textContent).toMatch(/menyusul/);
    expect(document.querySelector('input[type="file"]')).toBeNull();
  });

  test("validasi klien: isian salah → galat per kolom terhubung aria-describedby; API tidak dipanggil", async () => {
    const buat = vi.fn();
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ buat })} />);
    await isiForm({ "Email HR pertama": "bukan-email", NIB: "12 34" });
    kirim();
    const nib = await screen.findByLabelText("NIB");
    await waitFor(() => { expect(nib.getAttribute("aria-invalid")).toBe("true"); });
    expect(deskripsi(nib)).toContain("tidak boleh mengandung spasi");
    expect(deskripsi(screen.getByLabelText("Email HR pertama"))).toContain("email yang valid");
    expect(deskripsi(screen.getByLabelText("Nama badan usaha"))).toContain("wajib diisi");
    expect(buat).not.toHaveBeenCalled();
  });

  test("409 4090110 → galat pada NIB; 4090111 → galat pada email HR; formulir tetap terbuka dengan isian", async () => {
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ buat: () => gagal(409, "4090110") })} />);
    await isiForm();
    kirim();
    await waitFor(() => { expect(screen.getByLabelText("NIB").getAttribute("aria-invalid")).toBe("true"); });
    expect(deskripsi(screen.getByLabelText("NIB"))).toContain("NIB sudah terdaftar");
    expect(screen.getByLabelText<HTMLInputElement>("NIB").value).toBe(ISIAN.NIB);
    expect(screen.queryByText(/Perusahaan dibuat/)).toBeNull();
    cleanup();
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ buat: () => gagal(409, "4090111") })} />);
    await isiForm();
    kirim();
    await waitFor(() => { expect(screen.getByLabelText("Email HR pertama").getAttribute("aria-invalid")).toBe("true"); });
    expect(deskripsi(screen.getByLabelText("Email HR pertama"))).toContain("Email HR sudah dipakai");
    expect(screen.getByLabelText("NIB").getAttribute("aria-invalid")).toBeNull();
  });

  test("422 server → alert 'Isian tidak valid'; 403 4030101 → pesan muat ulang, bukan layar tanpa akses", async () => {
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ buat: () => gagal(422, "4220001") })} />);
    await isiForm();
    kirim();
    expect((await screen.findByRole("alert")).textContent).toContain("Isian tidak valid");
    cleanup();
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ buat: () => gagal(403, "4030101") })} />);
    await isiForm();
    kirim();
    expect((await screen.findByRole("alert")).textContent).toContain("Permintaan tidak dapat diverifikasi");
    expect(screen.queryByText("Tidak ada akses")).toBeNull();
    expect(screen.getByLabelText("NIB")).toBeTruthy();
  });

  test("negatif: 403 non-Verifikator saat membuat → layar tanpa akses, tanpa data dan tanpa formulir", async () => {
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ buat: () => gagal(403) })} />);
    await isiForm();
    kirim();
    expect(await screen.findByText("Tidak ada akses")).toBeTruthy();
    expect(screen.queryByLabelText("NIB")).toBeNull();
    expect(screen.queryByRole("table")).toBeNull();
    expect(document.body.textContent).not.toContain(ISIAN.NIB);
    expect(document.body.textContent).not.toContain("PT Sintetis Sejahtera");
  });

  test("tidak ada NIB, NPWP, atau email di console pada galat", async () => {
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ buat: () => gagal(409, "4090110") })} />);
    await isiForm();
    kirim();
    await waitFor(() => { expect(screen.getByLabelText("NIB").getAttribute("aria-invalid")).toBe("true"); });
    for (const s of Object.values(konsol)) {
      const teks = JSON.stringify(s.mock.calls);
      for (const rahasia of [ISIAN.NIB, ISIAN.NPWP, ISIAN["Email HR pertama"]]) expect(teks).not.toContain(rahasia);
    }
  });

  test("Batal menutup formulir tanpa memanggil API", async () => {
    const buat = vi.fn();
    render(<PanelPerusahaan csrfToken="csrf" klien={klienUji({ buat })} />);
    await isiForm();
    fireEvent.click(screen.getByRole("button", { name: "Batal" }));
    expect(screen.queryByLabelText("NIB")).toBeNull();
    expect(buat).not.toHaveBeenCalled();
  });

  test("klien: POST /bff/backoffice/v1/corporate-clients dengan badan terpangkas dan X-CSRF-Token; 201 diterima, 409 ke pesan dari kode", async () => {
    const dipanggil: { url: string; method: string; csrf: string | null; body: string }[] = [];
    let status = 201;
    const fetchUji = (async (r: Request) => {
      dipanggil.push({ url: new URL(r.url).pathname, method: r.method, csrf: r.headers.get("x-csrf-token"), body: await r.text() });
      return new Response(
        JSON.stringify(
          status === 201
            ? { response_code: "2010000", data: { corporate_client_id: ID, status: "active", corporate_member_id: ID, hr_account_id: ID, hr_invitation: "processing" } }
            : { response_code: "4090111", response_description: "<b>x</b> hr@perusahaan.example" },
        ),
        { status, headers: { "content-type": "application/json" } },
      );
    }) as unknown as typeof fetch;
    const k = buatKlienPerusahaan("tok", { basis: "http://bff.test/bff", fetch: fetchUji });
    const isian = { legal_name: " PT X ", nib: "1", npwp: "2", office_address: "a", office_phone: "3", hr_email: " hr@perusahaan.example " };
    const h = await k.buat(isian);
    expect(h).toMatchObject({ ok: true, data: { hr_invitation: "processing" } });
    expect(dipanggil[0]).toMatchObject({ url: "/bff/backoffice/v1/corporate-clients", method: "POST", csrf: "tok" });
    expect(JSON.parse(dipanggil[0]?.body ?? "")).toEqual({ ...isian, legal_name: "PT X", hr_email: "hr@perusahaan.example" });
    status = 409;
    const g = await k.buat(isian);
    expect(g).toMatchObject({ ok: false, status: 409, kode: "4090111" });
    expect(JSON.stringify(g)).not.toContain("hr@perusahaan.example");
  });
});
