import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { buatCsp, originUnggahDariEnv } from "../../keamanan.ts";
import { ITEM_NAV, menuUntuk } from "../../lib/navigasi.ts";
import { buatKlienVerifikasi, galat, type KlienVerifikasi } from "../../lib/verifikasi/klien.ts";
import { formatWib, nikTermasking, type Detail, type ItemAntrean } from "../../lib/verifikasi/model.ts";
import * as halaman from "../../app/(terautentikasi)/verifikasi/page.tsx";
import { PanelVerifikasi } from "./PanelVerifikasi.tsx";

const NIK = "9900000000000003";
const ID = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
const URL_FOTO = "https://neo.example.test/dok/abc?X-Sig=rahasia";

const item = (o: Partial<ItemAntrean> = {}): ItemAntrean => ({
  identity_verification_request_id: ID,
  care_recipient_id: "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2c",
  is_account_owner: false,
  full_name: "Citra",
  birth_date: "2018-02-03",
  sex: "female",
  document_type: "kk",
  trigger_type: "document_submitted",
  nik_last4: "0003",
  submitted_at: "2027-03-01T01:01:00Z",
  ...o,
});
const detail = (o: Partial<Detail> = {}): Detail => ({
  identity_verification_request_id: ID,
  care_recipient_id: "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2c",
  is_account_owner: false,
  full_name: "Citra",
  birth_date: "2018-02-03",
  sex: "female",
  document_type: "kk",
  trigger_type: "document_submitted",
  nik: NIK,
  submitted_at: "2027-03-01T01:01:00Z",
  status: "pending",
  can_decide: true,
  ...o,
});

const ok = <T,>(data: T) => Promise.resolve({ ok: true as const, data });
const gagal = (status: number, kode?: string) =>
  Promise.resolve(galat(status, kode === undefined ? {} : { response_code: kode }));

function klienUji(o: Partial<KlienVerifikasi> = {}): KlienVerifikasi {
  return {
    antrean: () => ok({ items: [item()], hasMore: false, nextCursor: null }),
    alasan: () =>
      ok([
        { code: "photo_unreadable", label: "Foto buram" },
        { code: "nik_unavailable", label: "NIK tidak tersedia" },
        { code: "other", label: "Lainnya" },
      ]),
    detail: () => ok(detail()),
    fotoUrl: () => ok({ url: URL_FOTO, expires_at: new Date(Date.now() + 60_000).toISOString(), mime_type: "image/jpeg", size_bytes: 10 }),
    setujui: () => ok({ identity_verification_request_id: ID, status: "approved" as const }),
    tolak: () => ok({ identity_verification_request_id: ID, status: "rejected" as const }),
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

async function bukaPanel(k: KlienVerifikasi) {
  render(<PanelVerifikasi csrfToken="csrf" klien={k} />);
  fireEvent.click(await screen.findByRole("button", { name: /Buka/ }));
  await screen.findByTestId("nik-lengkap");
}

describe("daftar antrean", () => {
  test("NIK termasking, jam berlabel WIB, tanpa NIK lengkap sebelum dibuka", async () => {
    render(<PanelVerifikasi csrfToken="csrf" klien={klienUji()} />);
    expect(await screen.findByText("************0003")).toBeTruthy();
    expect(screen.getByText(/1 Mar 2027, 08.01 WIB/)).toBeTruthy();
    expect(document.body.textContent).not.toContain(NIK);
  });

  test("antrean kosong", async () => {
    render(<PanelVerifikasi csrfToken="csrf" klien={klienUji({ antrean: () => ok({ items: [], hasMore: false, nextCursor: null }) })} />);
    expect(await screen.findByText("Antrean kosong")).toBeTruthy();
  });

  test("muat lebih banyak menambah baris", async () => {
    const antrean = vi
      .fn()
      .mockImplementationOnce(() => ok({ items: [item()], hasMore: true, nextCursor: "c1" }))
      .mockImplementationOnce(() => ok({ items: [item({ identity_verification_request_id: "x2", full_name: "Budi" })], hasMore: false, nextCursor: null }));
    render(<PanelVerifikasi csrfToken="csrf" klien={klienUji({ antrean })} />);
    fireEvent.click(await screen.findByRole("button", { name: "Muat lebih banyak" }));
    expect(await screen.findByText("Budi")).toBeTruthy();
    expect(antrean).toHaveBeenLastCalledWith("c1");
  });
});

describe("panel detail", () => {
  test("NIK lengkap hanya setelah dibuka; foto hanya setelah diminta, tanpa referrer dan dari URL bertanda tangan", async () => {
    const fotoUrl = vi.fn((id: string) => klienUji().fotoUrl(id));
    await bukaPanel(klienUji({ fotoUrl }));
    expect(screen.getByTestId("nik-lengkap").textContent).toContain(NIK);
    expect(document.querySelector("img")).toBeNull();
    expect(fotoUrl).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Tampilkan foto dokumen" }));
    await waitFor(() => { expect(document.querySelector("img")).not.toBeNull(); });
    const img = document.querySelector("img");
    expect(img?.getAttribute("referrerpolicy")).toBe("no-referrer");
    expect(img?.getAttribute("src")).toBe(URL_FOTO);
  });

  test("menutup panel membuang NIK lengkap dan foto dari DOM; tidak ada penyimpanan peramban", async () => {
    await bukaPanel(klienUji());
    fireEvent.click(screen.getByRole("button", { name: "Tampilkan foto dokumen" }));
    await waitFor(() => { expect(document.querySelector("img")).not.toBeNull(); });
    fireEvent.click(screen.getByRole("button", { name: "Tutup" }));
    expect(document.querySelector("img")).toBeNull();
    expect(document.body.textContent).not.toContain(NIK);
    expect(localStorage.length).toBe(0);
    expect(sessionStorage.length).toBe(0);
  });

  test("foto dilepas saat URL kedaluwarsa", async () => {
    const fotoUrl = () => ok({ url: URL_FOTO, expires_at: new Date(Date.now() + 50).toISOString(), mime_type: "image/jpeg", size_bytes: 1 });
    await bukaPanel(klienUji({ fotoUrl }));
    fireEvent.click(screen.getByRole("button", { name: "Tampilkan foto dokumen" }));
    await waitFor(() => { expect(document.querySelector("img")).toBeNull(); expect(screen.getByRole("button", { name: "Tampilkan foto dokumen" })).toBeTruthy(); }, { timeout: 2000 });
  });

  test("galat memuat gambar tidak menampilkan URL", async () => {
    await bukaPanel(klienUji());
    fireEvent.click(screen.getByRole("button", { name: "Tampilkan foto dokumen" }));
    await waitFor(() => { expect(document.querySelector("img")).not.toBeNull(); });
    fireEvent.error(screen.getByAltText("Foto KK pengajuan"));
    expect(await screen.findByText("Foto tidak dapat dimuat")).toBeTruthy();
    expect(document.body.innerHTML).not.toContain("rahasia");
  });

  test("pengajuan yang tidak lagi menunggu: tanpa tombol keputusan", async () => {
    await bukaPanel(klienUji({ detail: () => ok(detail({ can_decide: false, status: "approved" })) }));
    expect(screen.queryByRole("button", { name: "Setujui" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Tolak" })).toBeNull();
  });
});

describe("keputusan", () => {
  test("setujui memanggil API dan menyegarkan antrean", async () => {
    const setujui = vi.fn((id: string) => klienUji().setujui(id));
    const antrean = vi
      .fn()
      .mockImplementationOnce(() => ok({ items: [item()], hasMore: false, nextCursor: null }))
      .mockImplementationOnce(() => ok({ items: [], hasMore: false, nextCursor: null }));
    await bukaPanel(klienUji({ setujui, antrean }));
    fireEvent.click(screen.getByRole("button", { name: "Setujui" }));
    expect(await screen.findByText("Pengajuan disetujui. Pasien diberi tahu.")).toBeTruthy();
    expect(setujui).toHaveBeenCalledWith(ID);
    expect(await screen.findByText("Antrean kosong")).toBeTruthy();
    expect(document.body.textContent).not.toContain(NIK);
  });

  test("tolak tanpa alasan ditolak di klien; daftar alasan dari API", async () => {
    const tolak = vi.fn((id: string, k: string, t: string) => klienUji().tolak(id, k, t));
    await bukaPanel(klienUji({ tolak }));
    fireEvent.click(screen.getByRole("button", { name: "Tolak" }));
    expect(screen.getByText("Pilih alasan untuk menolak.")).toBeTruthy();
    expect(tolak).not.toHaveBeenCalled();
    const pilih = screen.getByLabelText("Alasan penolakan");
    expect(within(pilih).getAllByRole("option").map((o) => o.textContent)).toEqual(["Pilih alasan", "Foto buram", "NIK tidak tersedia", "Lainnya"]);
  });

  test("alasan Lainnya wajib keterangan; lalu terkirim dengan reason_code dan keterangan", async () => {
    const tolak = vi.fn((id: string, k: string, t: string) => klienUji().tolak(id, k, t));
    await bukaPanel(klienUji({ tolak }));
    fireEvent.change(screen.getByLabelText("Alasan penolakan"), { target: { value: "other" } });
    fireEvent.click(screen.getByRole("button", { name: "Tolak" }));
    expect(screen.getByText("Keterangan wajib diisi untuk alasan Lainnya.")).toBeTruthy();
    fireEvent.change(screen.getByLabelText(/Keterangan tambahan/), { target: { value: "Sudut terpotong" } });
    fireEvent.click(screen.getByRole("button", { name: "Tolak" }));
    await screen.findByText(/Pengajuan ditolak/);
    expect(tolak).toHaveBeenCalledWith(ID, "other", "Sudut terpotong");
  });

  test("4090204 sudah diputuskan: pesan, tombol hilang, antrean disegarkan", async () => {
    const antrean = vi.fn(() => klienUji().antrean());
    await bukaPanel(klienUji({ antrean, setujui: () => gagal(409, "4090204") }));
    fireEvent.click(screen.getByRole("button", { name: "Setujui" }));
    expect(await screen.findByText("Pengajuan sudah diputuskan")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Setujui" })).toBeNull();
    expect(antrean).toHaveBeenCalledTimes(2);
  });

  test("4090203 NIK terpakai: Setujui nonaktif dan alasan nik_unavailable terpilih", async () => {
    await bukaPanel(klienUji({ setujui: () => gagal(409, "4090203") }));
    fireEvent.click(screen.getByRole("button", { name: "Setujui" }));
    expect(await screen.findByText("NIK sudah dipakai")).toBeTruthy();
    expect(screen.getByLabelText<HTMLSelectElement>("Alasan penolakan").value).toBe("nik_unavailable");
    expect(screen.getByRole<HTMLButtonElement>("button", { name: "Setujui" }).disabled).toBe(true);
    expect(screen.getByRole<HTMLButtonElement>("button", { name: "Tolak" }).disabled).toBe(false);
  });

  test("4090205 pemisahan tugas: pesan dan tombol keputusan tidak tersedia", async () => {
    await bukaPanel(klienUji({ tolak: () => gagal(409, "4090205") }));
    fireEvent.change(screen.getByLabelText("Alasan penolakan"), { target: { value: "photo_unreadable" } });
    fireEvent.click(screen.getByRole("button", { name: "Tolak" }));
    expect(await screen.findByText(/pemisahan tugas/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Setujui" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Tolak" })).toBeNull();
  });
});

describe("negatif", () => {
  test("403 pada antrean: layar tanpa akses, tanpa data", async () => {
    render(<PanelVerifikasi csrfToken="csrf" klien={klienUji({ antrean: () => gagal(403), alasan: () => gagal(403) })} />);
    expect(await screen.findByText("Tidak ada akses")).toBeTruthy();
    expect(screen.queryByRole("table")).toBeNull();
  });

  test("503 pada antrean dan galat membuka detail tampil sebagai pesan", async () => {
    render(<PanelVerifikasi csrfToken="csrf" klien={klienUji({ antrean: () => gagal(503) })} />);
    expect(await screen.findByText("Layanan sedang tidak tersedia. Coba lagi nanti.")).toBeTruthy();
    cleanup();
    render(<PanelVerifikasi csrfToken="csrf" klien={klienUji({ detail: () => gagal(404) })} />);
    fireEvent.click(await screen.findByRole("button", { name: /Buka/ }));
    expect(await screen.findByText("Pengajuan tidak ditemukan.")).toBeTruthy();
  });

  test("NIK tidak bocor ke console pada alur sukses maupun galat", async () => {
    await bukaPanel(klienUji({ setujui: () => gagal(409, "4090205") }));
    fireEvent.click(screen.getByRole("button", { name: "Setujui" }));
    await screen.findByText(/pemisahan tugas/);
    for (const s of Object.values(konsol)) {
      expect(JSON.stringify(s.mock.calls)).not.toContain(NIK);
      expect(s).not.toHaveBeenCalled();
    }
  });
});

describe("klien dan galat", () => {
  test("galat memilih teks dari kode, bukan dari isi respons (isi tidak dipercaya)", () => {
    const g = galat(409, { response_code: "4090204", response_description: `NIK ${NIK} bocor` });
    expect(g.pesan).not.toContain(NIK);
    expect(g.kode).toBe("4090204");
    expect(galat(500, { response_description: NIK }).pesan).toBe("Permintaan gagal. Coba lagi.");
  });

  test("mutasi membawa X-CSRF-Token; foto lewat POST ke BFF; badan tolak memuat reason_code dan note", async () => {
    const dipanggil: { url: string; method: string; csrf: string | null; body: string }[] = [];
    const fetchUji = (async (r: Request) => {
      dipanggil.push({ url: new URL(r.url).pathname, method: r.method, csrf: r.headers.get("x-csrf-token"), body: await r.text() });
      return new Response(JSON.stringify({ response_code: "2000000", response_description: "Berhasil", data: { url: "u" } }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    }) as unknown as typeof fetch;
    const k = buatKlienVerifikasi("tok", { basis: "http://bff.test/bff", fetch: fetchUji });
    await k.setujui(ID);
    await k.tolak(ID, "other", " catatan ");
    await k.fotoUrl(ID);
    await k.detail(ID);
    expect(dipanggil.map((d) => `${d.method} ${d.url}`)).toEqual([
      `POST /bff/backoffice/v1/identity-verifications/${ID}/approve`,
      `POST /bff/backoffice/v1/identity-verifications/${ID}/reject`,
      `POST /bff/backoffice/v1/identity-verifications/${ID}/document-photo-url`,
      `GET /bff/backoffice/v1/identity-verifications/${ID}`,
    ]);
    expect(dipanggil.every((d) => d.csrf === "tok")).toBe(true);
    expect(JSON.parse(dipanggil[1]?.body ?? "")).toEqual({ reason_code: "other", note: "catatan" });
  });

  test("galat jaringan menjadi 503 terkendali", async () => {
    const k = buatKlienVerifikasi("t", { basis: "http://bff.test/bff", fetch: (() => Promise.reject(new Error("x"))) as unknown as typeof fetch });
    expect(await k.antrean()).toMatchObject({ ok: false, status: 503 });
  });
});

describe("model, navigasi, halaman, CSP", () => {
  test("formatWib berlabel WIB dan tidak bergantung zona peramban; nilai tak valid jadi tanda hubung", () => {
    expect(formatWib("2027-03-01T17:30:00Z")).toBe("2 Mar 2027, 00.30 WIB");
    expect(formatWib("bukan tanggal")).toBe("-");
    expect(formatWib(null)).toBe("-");
    expect(nikTermasking("0001")).toBe("************0001");
  });

  test("menu verifikasi hanya untuk sub-peran Verifikator", () => {
    expect(menuUntuk(["Verifikator"], ITEM_NAV).map((i) => i.href)).toContain("/verifikasi");
    for (const s of ["Konten", "Keuangan", "Layanan Pelanggan", "Auditor", "Petugas PDP", "Super Admin"]) {
      expect(menuUntuk([s], ITEM_NAV).map((i) => i.href)).not.toContain("/verifikasi");
    }
    expect(menuUntuk([], ITEM_NAV).map((i) => i.href)).not.toContain("/verifikasi");
  });

  test("halaman mengekspor komponen halaman", () => {
    expect(typeof halaman.default).toBe("function");
  });

  test("CSP: origin foto dokumen hanya di img-src, tidak melonggarkan sisanya", () => {
    const csp = buatCsp("n", false, null, "https://neo.example.test");
    expect(csp).toContain("img-src 'self' https://neo.example.test;");
    expect(csp).toContain("connect-src 'self';");
    expect(csp).not.toMatch(/unsafe-/);
    expect(buatCsp("n")).toContain("img-src 'self';");
    expect(originUnggahDariEnv("http://neo.example.test", false)).toBeNull();
  });
});
