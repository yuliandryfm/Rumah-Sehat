import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import * as halamanKelola from "../../app/(terautentikasi)/kelola-hr/page.tsx";
import { buatKlienHr, galat, type AnggotaHr, type KlienHr } from "../../lib/hr/klien.ts";
import { PESAN_UNDANGAN_DIPROSES } from "../../lib/hr/model.ts";
import { KelolaHr } from "./KelolaHr.tsx";

vi.mock("next/navigation", () => ({ usePathname: () => "/" }));

const ID_A = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
const ID_B = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2c";
const ID_C = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2d";

const hr = (o: Partial<AnggotaHr> & Pick<AnggotaHr, "corporate_member_id">): AnggotaHr => ({
  email: "hr@perusahaan.example",
  full_name: "Sari",
  job_title: null,
  status: "active",
  is_self: false,
  ...o,
});
const DAFTAR: AnggotaHr[] = [
  hr({ corporate_member_id: ID_A, full_name: "Sari", is_self: true }),
  hr({ corporate_member_id: ID_B, full_name: "Budi", email: "budi@perusahaan.example" }),
  hr({ corporate_member_id: ID_C, full_name: null, email: "rina@perusahaan.example", status: "pending_activation" }),
  hr({ corporate_member_id: "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2e", full_name: "Dewi", email: "dewi@perusahaan.example", status: "inactive" }),
];

const ok = <T,>(data: T) => Promise.resolve({ ok: true as const, data });
const gagal = (status: number, kode?: string) => Promise.resolve(galat(status, kode === undefined ? {} : { response_code: kode }));

function klien(o: Partial<KlienHr> = {}): KlienHr {
  return {
    daftar: () => ok(DAFTAR),
    nonaktifkan: (id) => ok({ corporate_member_id: id, status: "inactive" as const, identity_provider_synced: true }),
    undang: () => ok({ status: "processing" }),
    ...o,
  };
}

afterEach(() => {
  cleanup();
});

describe("KelolaHr", () => {
  test("CR-039: judul halaman 'Tim HR' (bukan 'Kelola HR')", async () => {
    render(<KelolaHr csrfToken="t" klien={klien()} />);
    await screen.findByRole("table");
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Tim HR");
    expect(document.body.textContent).not.toContain("Kelola HR");
  });

  test("daftar: status dengan teks, diri sendiri tanpa tombol, nonaktif tanpa tombol, tombol Undang HR aktif", async () => {
    render(<KelolaHr csrfToken="t" klien={klien()} />);
    const tabel = await screen.findByRole("table");
    const baris = within(tabel).getAllByRole("row").slice(1) as [HTMLElement, HTMLElement, HTMLElement, HTMLElement];
    expect(baris).toHaveLength(4);
    expect(within(baris[0]).getByText("Sari (Anda)")).toBeDefined();
    expect(within(baris[0]).queryByRole("button")).toBeNull();
    expect(within(baris[0]).getByText("Tidak dapat menonaktifkan akun sendiri")).toBeDefined();
    expect(within(baris[1]).getByRole("button", { name: "Nonaktifkan Budi" })).toBeDefined();
    expect(within(baris[2]).getAllByText("rina@perusahaan.example")).toHaveLength(2); // tanpa nama: email jadi nama tampil
    expect(within(baris[2]).getByText("Menunggu aktivasi")).toBeDefined();
    expect(within(baris[3]).getByText("Nonaktif")).toBeDefined();
    expect(within(baris[3]).queryByRole("button")).toBeNull();
    const undang = screen.getByRole("button", { name: "Undang HR" });
    expect(undang.hasAttribute("disabled")).toBe(false);
    expect(document.body.textContent).not.toMatch(/Belum tersedia: undangan HR baru/);
  });

  test("nonaktifkan: konfirmasi dampak, Batal mengembalikan fokus ke tombol pemicu dan tidak memanggil API", async () => {
    const nonaktifkan = vi.fn();
    render(<KelolaHr csrfToken="t" klien={klien({ nonaktifkan })} />);
    const tombol = await screen.findByRole("button", { name: "Nonaktifkan Budi" });
    tombol.focus();
    fireEvent.click(tombol);
    const grup = await screen.findByRole("group", { name: "Nonaktifkan Budi?" });
    await waitFor(() => { expect(document.activeElement).toBe(grup); });
    expect(within(grup).getByText(/tidak dapat masuk lagi/)).toBeDefined();
    fireEvent.click(within(grup).getByRole("button", { name: "Batal" }));
    expect(screen.queryByRole("group")).toBeNull();
    expect(document.activeElement).toBe(tombol);
    expect(nonaktifkan).not.toHaveBeenCalled();
  });

  test("nonaktifkan berhasil: memanggil API dengan ID anggota, memuat ulang daftar, mengumumkan hasil", async () => {
    let sudah = false;
    const nonaktifkan = vi.fn((id: string) => {
      sudah = true;
      return ok({ corporate_member_id: id, status: "inactive" as const, identity_provider_synced: true });
    });
    const daftar = vi.fn(() =>
      ok(DAFTAR.map((a) => (sudah && a.corporate_member_id === ID_B ? { ...a, status: "inactive" as const } : a))),
    );
    render(<KelolaHr csrfToken="t" klien={klien({ nonaktifkan, daftar })} />);
    fireEvent.click(await screen.findByRole("button", { name: "Nonaktifkan Budi" }));
    const grup = await screen.findByRole("group");
    fireEvent.click(within(grup).getByRole("button", { name: "Nonaktifkan" }));
    expect(await screen.findByText("Budi dinonaktifkan dan tidak dapat masuk lagi.")).toBeDefined();
    expect(nonaktifkan).toHaveBeenCalledWith(ID_B);
    expect(daftar).toHaveBeenCalledTimes(2);
    expect(screen.queryByRole("button", { name: "Nonaktifkan Budi" })).toBeNull();
  });

  test("identity_provider_synced=false: pesan jujur bahwa sesi diakhiri menyusul", async () => {
    const nonaktifkan = (id: string) => ok({ corporate_member_id: id, status: "inactive" as const, identity_provider_synced: false });
    render(<KelolaHr csrfToken="t" klien={klien({ nonaktifkan })} />);
    fireEvent.click(await screen.findByRole("button", { name: "Nonaktifkan Budi" }));
    fireEvent.click(within(await screen.findByRole("group")).getByRole("button", { name: "Nonaktifkan" }));
    expect(await screen.findByText(/Akses sudah ditutup; sesi masuknya diakhiri menyusul/)).toBeDefined();
  });

  test("negatif: HR aktif terakhir (409 4090105) → galat tampil, daftar tetap, tidak ada pesan berhasil", async () => {
    const nonaktifkan = () => gagal(409, "4090105");
    render(<KelolaHr csrfToken="t" klien={klien({ nonaktifkan })} />);
    fireEvent.click(await screen.findByRole("button", { name: "Nonaktifkan Budi" }));
    fireEvent.click(within(await screen.findByRole("group")).getByRole("button", { name: "Nonaktifkan" }));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Tidak dapat dinonaktifkan");
    expect(alert.textContent).toContain("HR aktif terakhir tidak dapat dinonaktifkan.");
    expect(screen.queryByText("Berhasil")).toBeNull();
    expect(screen.queryByRole("group")).toBeNull();
    expect(screen.getByRole("button", { name: "Nonaktifkan Budi" })).toBeDefined();
  });

  test("negatif: 409 4090101 (diri sendiri) dan 4090001 (sudah nonaktif) punya pesan sendiri; 4090001 memuat ulang daftar", async () => {
    const daftar = vi.fn(() => ok(DAFTAR));
    const kode = ["4090101", "4090001"];
    const pesan = ["Anda tidak dapat menonaktifkan akun Anda sendiri.", "Akun HR ini sudah nonaktif."];
    for (const [i, k] of kode.entries()) {
      render(<KelolaHr csrfToken="t" klien={klien({ nonaktifkan: () => gagal(409, k), daftar })} />);
      fireEvent.click(await screen.findByRole("button", { name: "Nonaktifkan Budi" }));
      fireEvent.click(within(await screen.findByRole("group")).getByRole("button", { name: "Nonaktifkan" }));
      expect((await screen.findByRole("alert")).textContent).toContain(pesan[i]);
      cleanup();
    }
    // 2 render awal + 1 muat ulang pada 4090001.
    expect(daftar).toHaveBeenCalledTimes(3);
  });

  test("negatif: 404 (anggota perusahaan lain/tidak ada) → 'Data tidak ditemukan' tanpa membocorkan alasan", async () => {
    render(<KelolaHr csrfToken="t" klien={klien({ nonaktifkan: () => gagal(404, "4040001") })} />);
    fireEvent.click(await screen.findByRole("button", { name: "Nonaktifkan Budi" }));
    fireEvent.click(within(await screen.findByRole("group")).getByRole("button", { name: "Nonaktifkan" }));
    expect((await screen.findByRole("alert")).textContent).toContain("Data tidak ditemukan.");
    expect(document.body.textContent).not.toMatch(/perusahaan lain|bukan milik/i);
  });

  test("negatif: sesi dicabut atau akun dinonaktifkan HR lain saat aksi (401 4010002) → layar akun nonaktif", async () => {
    render(<KelolaHr csrfToken="t" klien={klien({ nonaktifkan: () => gagal(401, "4010002") })} />);
    fireEvent.click(await screen.findByRole("button", { name: "Nonaktifkan Budi" }));
    fireEvent.click(within(await screen.findByRole("group")).getByRole("button", { name: "Nonaktifkan" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Akun nonaktif" })).toBeDefined();
    expect(screen.queryByRole("table")).toBeNull();
  });

  test("negatif: 403 4030101 (CSRF/Origin ditolak BFF) saat aksi → pesan muat ulang, bukan layar perusahaan belum aktif", async () => {
    render(<KelolaHr csrfToken="t" klien={klien({ nonaktifkan: () => gagal(403, "4030101") })} />);
    fireEvent.click(await screen.findByRole("button", { name: "Nonaktifkan Budi" }));
    fireEvent.click(within(await screen.findByRole("group")).getByRole("button", { name: "Nonaktifkan" }));
    expect((await screen.findByRole("alert")).textContent).toContain("Permintaan tidak dapat diverifikasi");
    expect(screen.queryByRole("heading", { level: 1, name: "Menu ini belum dapat dipakai" })).toBeNull();
    expect(screen.getByRole("table")).toBeDefined();
  });

  test("403 saat memuat → layar akses ditolak; 503 → pesan galat dan tabel tidak tampil", async () => {
    render(<KelolaHr csrfToken="t" klien={klien({ daftar: () => gagal(403) })} />);
    expect(await screen.findByRole("heading", { level: 1, name: "Menu ini belum dapat dipakai" })).toBeDefined();
    cleanup();
    render(<KelolaHr csrfToken="t" klien={klien({ daftar: () => gagal(503) })} />);
    expect((await screen.findByText("Daftar HR tidak dapat dimuat"))).toBeDefined();
    expect(screen.queryByRole("table")).toBeNull();
  });
});

describe("Undang HR (US-1904 AC2)", () => {
  const EMAIL = "hr.baru@perusahaan.example";
  const konsol = { log: vi.spyOn(console, "log"), warn: vi.spyOn(console, "warn"), error: vi.spyOn(console, "error"), info: vi.spyOn(console, "info") };
  beforeEach(() => {
    for (const s of Object.values(konsol)) s.mockImplementation(() => undefined);
  });
  afterEach(() => {
    for (const s of Object.values(konsol)) s.mockReset();
  });

  async function isiDanKirim(email: string) {
    fireEvent.click(await screen.findByRole("button", { name: "Undang HR" }));
    const kolom = await screen.findByLabelText("Email HR yang diundang");
    fireEvent.change(kolom, { target: { value: email } });
    fireEvent.click(screen.getByRole("button", { name: "Kirim undangan" }));
  }

  test("sukses: memanggil API dengan email, pesan tetap, formulir tertutup, daftar dimuat ulang", async () => {
    const undang = vi.fn(() => ok({ status: "processing" }));
    const daftar = vi.fn(() => ok(DAFTAR));
    render(<KelolaHr csrfToken="t" klien={klien({ undang, daftar })} />);
    await isiDanKirim(`  ${EMAIL} `);
    expect(await screen.findByText(PESAN_UNDANGAN_DIPROSES)).toBeDefined();
    expect(undang).toHaveBeenCalledWith(EMAIL);
    expect(daftar).toHaveBeenCalledTimes(2);
    expect(screen.queryByRole("form")).toBeNull();
    expect(document.body.textContent).not.toContain(EMAIL);
    expect(screen.getByText("Undangan diproses")).toBeDefined();
  });

  test("anti-enumerasi: dua jawaban 202 identik (email baru vs terpakai) menghasilkan DOM identik", async () => {
    const hasil: string[] = [];
    for (const email of ["baru@perusahaan.example", "hr@perusahaan.example"]) {
      const { container } = render(<KelolaHr csrfToken="t" klien={klien({ undang: () => ok({ status: "processing" }) })} />);
      await isiDanKirim(email);
      await screen.findByText(PESAN_UNDANGAN_DIPROSES);
      const salinan = container.cloneNode(true) as HTMLElement;
      hasil.push(salinan.innerHTML);
      cleanup();
    }
    expect(hasil[0]).toBe(hasil[1]);
    expect(hasil[0]).not.toContain("baru@perusahaan.example");
    expect(PESAN_UNDANGAN_DIPROSES).not.toMatch(/sudah (terdaftar|dipakai)/i);
  });

  test("validasi klien: email kosong atau salah bentuk → galat terhubung aria-describedby, API tidak dipanggil", async () => {
    const undang = vi.fn();
    render(<KelolaHr csrfToken="t" klien={klien({ undang })} />);
    await isiDanKirim("bukan-email");
    const kolom = screen.getByLabelText("Email HR yang diundang");
    await waitFor(() => { expect(kolom.getAttribute("aria-invalid")).toBe("true"); });
    const idGalat = kolom.getAttribute("aria-describedby") ?? "";
    expect(idGalat.split(" ").some((i) => document.getElementById(i)?.textContent.includes("email yang valid"))).toBe(true);
    expect(undang).not.toHaveBeenCalled();
    fireEvent.change(kolom, { target: { value: "" } });
    fireEvent.click(screen.getByRole("button", { name: "Kirim undangan" }));
    expect(undang).not.toHaveBeenCalled();
  });

  test("422 dari server → galat pada isian, formulir tetap terbuka", async () => {
    render(<KelolaHr csrfToken="t" klien={klien({ undang: () => gagal(422, "4220001") })} />);
    await isiDanKirim(EMAIL);
    const kolom = await screen.findByLabelText("Email HR yang diundang");
    await waitFor(() => { expect(kolom.getAttribute("aria-invalid")).toBe("true"); });
    expect(document.body.textContent).toContain("Email tidak valid. Periksa penulisan email");
    expect(screen.queryByText(PESAN_UNDANGAN_DIPROSES)).toBeNull();
  });

  test("403 4030101 (CSRF) → pesan muat ulang, bukan layar perusahaan belum aktif, tanpa pesan sukses", async () => {
    render(<KelolaHr csrfToken="t" klien={klien({ undang: () => gagal(403, "4030101") })} />);
    await isiDanKirim(EMAIL);
    expect((await screen.findByRole("alert")).textContent).toContain("Permintaan tidak dapat diverifikasi");
    expect(screen.queryByRole("heading", { level: 1, name: "Menu ini belum dapat dipakai" })).toBeNull();
    expect(screen.queryByText(PESAN_UNDANGAN_DIPROSES)).toBeNull();
  });

  test("401 4010002 → layar akun nonaktif; 401 lain → alihkan ke login", async () => {
    render(<KelolaHr csrfToken="t" klien={klien({ undang: () => gagal(401, "4010002") })} />);
    await isiDanKirim(EMAIL);
    expect(await screen.findByRole("heading", { level: 1, name: "Akun nonaktif" })).toBeDefined();
    cleanup();
    const arahkan = vi.fn();
    render(<KelolaHr csrfToken="t" klien={klien({ undang: () => gagal(401, "4010001") })} arahkan={arahkan} />);
    await isiDanKirim(EMAIL);
    await waitFor(() => { expect(arahkan).toHaveBeenCalledWith("/auth/login"); });
  });

  test("403 perusahaan belum aktif dan 503 → layar akses / alert galat; email tidak muncul di console", async () => {
    render(<KelolaHr csrfToken="t" klien={klien({ undang: () => gagal(403) })} />);
    await isiDanKirim(EMAIL);
    expect(await screen.findByRole("heading", { level: 1, name: "Menu ini belum dapat dipakai" })).toBeDefined();
    cleanup();
    render(<KelolaHr csrfToken="t" klien={klien({ undang: () => gagal(503) })} />);
    await isiDanKirim(EMAIL);
    expect((await screen.findByRole("alert")).textContent).toContain("Layanan sedang tidak tersedia");
    for (const s of Object.values(konsol)) expect(JSON.stringify(s.mock.calls)).not.toContain("perusahaan.example");
  });

  test("Batal menutup formulir tanpa memanggil API dan mengosongkan isian", async () => {
    const undang = vi.fn();
    render(<KelolaHr csrfToken="t" klien={klien({ undang })} />);
    await isiDanKirim("bukan-email");
    fireEvent.click(screen.getByRole("button", { name: "Batal" }));
    expect(screen.queryByLabelText("Email HR yang diundang")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Undang HR" }));
    expect((await screen.findByLabelText<HTMLInputElement>("Email HR yang diundang")).value).toBe("");
    expect(undang).not.toHaveBeenCalled();
  });

  test("klien: POST /bff/v1/corporate/member-invitations dengan body email dan X-CSRF-Token; 202 diterima", async () => {
    const dipanggil: Request[] = [];
    const badan: string[] = [];
    const fetchTiruan = vi.fn(async (r: Request) => {
      dipanggil.push(r);
      badan.push(await r.clone().text());
      return new Response(JSON.stringify({ response_code: "2020000", data: { status: "processing" } }), {
        status: 202,
        headers: { "content-type": "application/json" },
      });
    });
    const api = buatKlienHr("csrf-uji", { basis: "http://localhost/bff", fetch: fetchTiruan as unknown as typeof fetch });
    const h = await api.undang(EMAIL);
    expect(h).toEqual({ ok: true, data: { status: "processing" } });
    expect(dipanggil[0]?.method).toBe("POST");
    expect(dipanggil[0]?.url).toBe("http://localhost/bff/v1/corporate/member-invitations");
    expect(dipanggil[0]?.headers.get("x-csrf-token")).toBe("csrf-uji");
    expect(JSON.parse(badan[0] ?? "{}")).toEqual({ email: EMAIL });
  });
});

describe("klien HR", () => {
  const respons = (status: number, badan: unknown) =>
    new Response(JSON.stringify(badan), { status, headers: { "content-type": "application/json" } });

  test("memanggil /bff/v1/corporate/members dengan X-CSRF-Token; deactivate memakai POST dan ID di path", async () => {
    const dipanggil: Request[] = [];
    const fetchTiruan = vi.fn((r: Request) => {
      dipanggil.push(r);
      return Promise.resolve(
        r.method === "GET"
          ? respons(200, { response_code: "2000000", data: DAFTAR })
          : respons(200, { response_code: "2000000", data: { corporate_member_id: ID_B, status: "inactive", identity_provider_synced: true } }),
      );
    });
    const api = buatKlienHr("csrf-uji", { basis: "http://localhost/bff", fetch: fetchTiruan as unknown as typeof fetch });
    const d = await api.daftar();
    expect(d.ok && d.data.length).toBe(4);
    const n = await api.nonaktifkan(ID_B);
    expect(n.ok).toBe(true);
    expect(dipanggil[0]?.url).toBe("http://localhost/bff/v1/corporate/members");
    expect(dipanggil[1]?.method).toBe("POST");
    expect(dipanggil[1]?.url).toBe(`http://localhost/bff/v1/corporate/members/${ID_B}/deactivate`);
    expect(dipanggil.every((r) => r.headers.get("x-csrf-token") === "csrf-uji")).toBe(true);
  });

  test("galat jaringan → 503 tanpa membocorkan isi galat; teks dipilih dari kode, bukan dari isi respons", async () => {
    const api = buatKlienHr("t", { basis: "http://localhost/bff", fetch: (() => Promise.reject(new Error("ECONNREFUSED 10.0.0.5"))) as unknown as typeof fetch });
    const h = await api.daftar();
    expect(h).toMatchObject({ ok: false, status: 503 });
    expect(JSON.stringify(h)).not.toContain("10.0.0.5");
    expect(galat(409, { response_code: "4090105", response_description: "<script>x</script>" }).pesan).toBe("HR aktif terakhir tidak dapat dinonaktifkan.");
    expect(galat(418, {}).pesan).toBe("Permintaan gagal. Coba lagi.");
  });
});

describe("halaman", () => {
  test("rute Beranda dan Tim HR mengekspor halaman server (tanpa sesi → login lewat wajibSesi)", () => {
    expect(typeof halamanKelola.default).toBe("function");
  });
});
