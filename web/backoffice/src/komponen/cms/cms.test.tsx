import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import type { KlienCms } from "../../lib/cms/klien.ts";
import { galat } from "../../lib/cms/klien.ts";
import type { Konten } from "../../lib/cms/model.ts";
import { DaftarKonten } from "./DaftarKonten.tsx";
import { EditorKonten } from "./EditorKonten.tsx";
import { PratinjauAman } from "./PratinjauAman.tsx";

const router = vi.hoisted(() => ({ replace: vi.fn() }));
vi.mock("next/navigation", () => ({ usePathname: () => "/konten", useRouter: () => router }));

afterEach(() => {
  cleanup();
  router.replace.mockReset();
});

const ID = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
const konten = (o: Partial<Konten> = {}): Konten => ({
  content_id: ID,
  title: "Tentang Kami",
  content_type: "info_page",
  channel: "site",
  site_page: "about",
  display_status: "draft",
  editorial_status: "draft",
  body: "## Siapa kami",
  created_at: "2027-01-01T00:00:00Z",
  updated_at: "2027-01-01T00:00:00Z",
  ...o,
});

const ok = <T,>(data: T) => Promise.resolve({ ok: true as const, data });
const gagal = (status: number, body: unknown = {}) => Promise.resolve(galat(status, body));

function klienUji(o: Partial<KlienCms> = {}): KlienCms {
  const tak = () => Promise.reject(new Error("tidak diharapkan dipanggil"));
  return {
    daftar: () => ok({ items: [], hasMore: false, nextCursor: null }),
    ambil: () => ok(konten()),
    buat: tak,
    ubah: tak,
    jadwalkan: tak,
    arsipkan: tak,
    hapus: tak,
    pulihkan: tak,
    unggahGambar: tak,
    ...o,
  };
}

const siap = () =>
  waitFor(() => {
    expect(screen.getByLabelText<HTMLInputElement>("Judul").value).toBe("Tentang Kami");
  });

describe("PratinjauAman (e2e jsdom: payload XSS)", () => {
  const PAYLOAD = '<script>window.__xss=1</script><img src=x onerror="window.__xss=1">[a](javascript:window.__xss=1)';

  test("iframe sandbox tanpa allow-same-origin dan tanpa allow-scripts; payload hanya teks di srcdoc", () => {
    render(<PratinjauAman judul={PAYLOAD} markdown={PAYLOAD} />);
    const f = screen.getByTitle("Pratinjau konten");
    expect(f.tagName).toBe("IFRAME");
    expect(f.hasAttribute("sandbox")).toBe(true);
    const sb = f.getAttribute("sandbox") ?? "";
    expect(sb).not.toContain("allow-same-origin");
    expect(sb).not.toContain("allow-scripts");
    expect(f.getAttribute("referrerpolicy")).toBe("no-referrer");
    expect(f.hasAttribute("src")).toBe(false);

    // Parse srcdoc seperti browser: tidak boleh ada elemen berbahaya dari payload.
    const doc = new DOMParser().parseFromString(f.getAttribute("srcdoc") ?? "", "text/html");
    expect(doc.querySelectorAll("script, img, iframe, a, object, embed").length).toBe(0);
    expect(doc.querySelector("pre")?.textContent).toBe(PAYLOAD);
    expect(doc.querySelector("h1")?.textContent).toBe(PAYLOAD);
    expect((window as unknown as { __xss?: number }).__xss).toBeUndefined();
  });
});

describe("DaftarKonten", () => {
  test("403 menampilkan layar Tidak ada akses", async () => {
    render(<DaftarKonten csrfToken="c" klien={klienUji({ daftar: () => gagal(403) })} />);
    expect(await screen.findByText("Tidak ada akses")).toBeDefined();
    expect(screen.queryByRole("table")).toBeNull();
  });

  test("daftar kosong: Tidak ada konten", async () => {
    render(<DaftarKonten csrfToken="c" klien={klienUji()} />);
    expect(await screen.findByText("Tidak ada konten")).toBeDefined();
  });

  test("baris menampilkan kanal, halaman situs, status; judul berisi HTML tampil sebagai teks", async () => {
    const jahat = '<img src=x onerror=alert(1)>';
    render(
      <DaftarKonten
        csrfToken="c"
        klien={klienUji({ daftar: () => ok({ items: [konten({ title: jahat, display_status: "scheduled" })], hasMore: false, nextCursor: null }) })}
      />,
    );
    const baris = await screen.findByRole("row", { name: /img/ });
    expect(within(baris).getByText(jahat)).toBeDefined();
    expect(baris.querySelector("img")).toBeNull();
    expect(within(baris).getByText("Situs")).toBeDefined();
    expect(within(baris).getByText("Tentang Kami")).toBeDefined();
    expect(within(baris).getByText("Tayang (terjadwal)")).toBeDefined();
  });

  test("saringan diteruskan ke API", async () => {
    const daftar = vi.fn(() => ok({ items: [], hasMore: false, nextCursor: null }));
    render(<DaftarKonten csrfToken="c" klien={klienUji({ daftar })} />);
    await screen.findByText("Tidak ada konten");
    fireEvent.change(screen.getByLabelText("Kanal"), { target: { value: "site" } });
    fireEvent.change(screen.getByLabelText("Status"), { target: { value: "deleted" } });
    await waitFor(() => { expect(daftar).toHaveBeenLastCalledWith({ status: "deleted", channel: "site" }); });
  });

  test("hapus: alasan wajib (kosong ditolak tanpa memanggil API), lalu terkirim dengan alasan", async () => {
    const hapus = vi.fn(() => ok({ content_id: ID }));
    render(
      <DaftarKonten csrfToken="c" klien={klienUji({ hapus, daftar: () => ok({ items: [konten()], hasMore: false, nextCursor: null }) })} />,
    );
    fireEvent.click(await screen.findByRole("button", { name: "Hapus" }));
    const dialog = screen.getByRole("alertdialog");
    fireEvent.click(within(dialog).getByRole("button", { name: "Hapus" }));
    expect(within(dialog).getByText(/Alasan wajib diisi/)).toBeDefined();
    expect(hapus).not.toHaveBeenCalled();
    fireEvent.change(within(dialog).getByLabelText("Alasan (wajib)"), { target: { value: "  konten usang " } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Hapus" }));
    await waitFor(() => { expect(hapus).toHaveBeenCalledWith(ID, "konten usang"); });
    expect(await screen.findByText(/dapat dipulihkan/)).toBeDefined();
  });

  test("filter Terhapus: kolom alasan dan tombol Pulihkan", async () => {
    const pulihkan = vi.fn(() => ok(konten()));
    const daftar = vi.fn(() =>
      ok({ items: [konten({ display_status: "deleted", deleted_at: "2027-01-02T00:00:00Z", delete_reason: "konten usang" })], hasMore: false, nextCursor: null }),
    );
    render(<DaftarKonten csrfToken="c" klien={klienUji({ daftar, pulihkan })} />);
    fireEvent.change(await screen.findByLabelText("Status"), { target: { value: "deleted" } });
    expect(await screen.findByText("konten usang")).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Pulihkan" }));
    await waitFor(() => { expect(pulihkan).toHaveBeenCalledWith(ID); });
  });
});

describe("EditorKonten", () => {
  test("konten baru: simpan memanggil buat lalu pindah ke editor; judul kosong ditolak", async () => {
    const buat = vi.fn(() => ok(konten()));
    render(<EditorKonten csrfToken="c" klien={klienUji({ buat })} />);
    fireEvent.click(screen.getByRole("button", { name: "Simpan draf" }));
    expect(await screen.findByText("Judul wajib diisi (1-200 karakter).")).toBeDefined();
    expect(buat).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText("Judul"), { target: { value: "Tentang Kami" } });
    fireEvent.change(screen.getByLabelText("Isi (Markdown)"), { target: { value: "## Siapa kami" } });
    fireEvent.click(screen.getByRole("button", { name: "Simpan draf" }));
    await waitFor(() => { expect(router.replace).toHaveBeenCalledWith(`/konten/${ID}`); });
    expect(buat).toHaveBeenCalledWith(expect.objectContaining({ title: "Tentang Kami", channel: "site", site_page: "about", body: "## Siapa kami" }));
  });

  test("pratinjau mengikuti isi yang diketik dan tetap sandbox", async () => {
    render(<EditorKonten csrfToken="c" kontenId={ID} klien={klienUji()} />);
    await siap();
    fireEvent.change(screen.getByLabelText("Isi (Markdown)"), { target: { value: "<script>alert(1)</script>" } });
    const f = screen.getByTitle("Pratinjau konten");
    expect(f.getAttribute("srcdoc")).toContain("&lt;script&gt;alert(1)&lt;/script&gt;");
    expect(f.getAttribute("sandbox")).toBe("");
  });

  test("galat per baris dari API ditampilkan sebagai teks, bukan HTML", async () => {
    const pesan = 'HTML mentah tidak diizinkan: <script>alert(1)</script>';
    const ubah = vi.fn(() => gagal(422, { errors: [{ field: "body", code: "raw_html", message: pesan }] }));
    render(<EditorKonten csrfToken="c" kontenId={ID} klien={klienUji({ ubah })} />);
    await siap();
    fireEvent.click(screen.getByRole("button", { name: "Simpan draf" }));
    const teks = await screen.findByText(pesan);
    expect(teks.querySelector("script")).toBeNull();
    expect(document.querySelector(".rs-field__error script")).toBeNull();
  });

  test("tayangkan: dialog konfirmasi menyebut versi lama berakhir; menyimpan lalu menjadwalkan", async () => {
    const ubah = vi.fn(() => ok(konten()));
    const jadwalkan = vi.fn(() => ok(konten({ display_status: "live", editorial_status: "scheduled" })));
    render(<EditorKonten csrfToken="c" kontenId={ID} klien={klienUji({ ubah, jadwalkan })} />);
    await siap();
    fireEvent.click(screen.getByRole("button", { name: "Tayangkan" }));
    const dialog = screen.getByRole("alertdialog");
    expect(within(dialog).getByText(/berakhir dan digantikan/)).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "Tayangkan" }));
    await waitFor(() => { expect(jadwalkan).toHaveBeenCalledWith(ID, null, null); });
    expect(ubah).toHaveBeenCalledOnce();
    expect(await screen.findByText("Konten ditayangkan.")).toBeDefined();
  });

  test("jadwal: waktu lokal dikirim sebagai ISO dengan offset WIB", async () => {
    const jadwalkan = vi.fn(() => ok(konten({ display_status: "scheduled" })));
    render(<EditorKonten csrfToken="c" kontenId={ID} klien={klienUji({ ubah: () => ok(konten()), jadwalkan })} />);
    await siap();
    fireEvent.change(screen.getByLabelText("Mulai tayang"), { target: { value: "2027-03-01T08:00" } });
    fireEvent.click(screen.getByRole("button", { name: "Jadwalkan" }));
    fireEvent.click(within(screen.getByRole("alertdialog")).getByRole("button", { name: "Jadwalkan" }));
    await waitFor(() => { expect(jadwalkan).toHaveBeenCalledWith(ID, "2027-03-01T08:00:00+07:00", null); });
  });

  test("arsipkan memanggil API", async () => {
    const arsipkan = vi.fn(() => ok(konten({ display_status: "archived", editorial_status: "archived" })));
    render(<EditorKonten csrfToken="c" kontenId={ID} klien={klienUji({ arsipkan })} />);
    await siap();
    fireEvent.click(screen.getByRole("button", { name: "Arsipkan" }));
    await waitFor(() => { expect(arsipkan).toHaveBeenCalledWith(ID); });
    expect(await screen.findByText(/diarsipkan/)).toBeDefined();
  });

  test("hapus dari editor: alasan wajib lalu kembali ke daftar", async () => {
    const hapus = vi.fn(() => ok({ content_id: ID }));
    render(<EditorKonten csrfToken="c" kontenId={ID} klien={klienUji({ hapus })} />);
    await siap();
    fireEvent.click(screen.getByRole("button", { name: "Hapus" }));
    const dialog = screen.getByRole("alertdialog");
    fireEvent.click(within(dialog).getByRole("button", { name: "Hapus" }));
    expect(hapus).not.toHaveBeenCalled();
    fireEvent.change(within(dialog).getByLabelText("Alasan (wajib)"), { target: { value: "usang" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Hapus" }));
    await waitFor(() => { expect(router.replace).toHaveBeenCalledWith("/konten"); });
    expect(hapus).toHaveBeenCalledWith(ID, "usang");
  });

  test("gambar: teks alternatif wajib sebelum unggah (API tidak dipanggil)", async () => {
    const unggahGambar = vi.fn(() => ok(konten()));
    render(<EditorKonten csrfToken="c" kontenId={ID} klien={klienUji({ unggahGambar })} />);
    await siap();
    const berkas = new File([new Uint8Array(3)], "a.png", { type: "image/png" });
    fireEvent.change(screen.getByLabelText(/Berkas gambar/), { target: { files: [berkas] } });
    fireEvent.click(screen.getByRole("button", { name: "Unggah dan pasang gambar" }));
    expect(await screen.findByText(/Teks alternatif wajib diisi/)).toBeDefined();
    expect(unggahGambar).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText("Teks alternatif gambar"), { target: { value: "Ilustrasi dokter" } });
    fireEvent.click(screen.getByRole("button", { name: "Unggah dan pasang gambar" }));
    await waitFor(() => { expect(unggahGambar).toHaveBeenCalledWith(ID, berkas, "Ilustrasi dokter"); });
  });

  test("konten tayang tidak dapat diubah: isian terkunci, tanpa Simpan/Tayangkan", async () => {
    render(<EditorKonten csrfToken="c" kontenId={ID} klien={klienUji({ ambil: () => ok(konten({ display_status: "live" })) })} />);
    expect(await screen.findByText("Konten tidak dapat diubah")).toBeDefined();
    expect(screen.getByLabelText<HTMLInputElement>("Judul").disabled).toBe(true);
    expect(screen.queryByRole("button", { name: "Simpan draf" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Tayangkan" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Arsipkan" })).toBeNull();
  });

  test("403 pada muat: Tidak ada akses; 404: Konten tidak ditemukan", async () => {
    render(<EditorKonten csrfToken="c" kontenId={ID} klien={klienUji({ ambil: () => gagal(403) })} />);
    expect(await screen.findByText("Tidak ada akses")).toBeDefined();
    cleanup();
    render(<EditorKonten csrfToken="c" kontenId={ID} klien={klienUji({ ambil: () => gagal(404) })} />);
    expect(await screen.findByText("Konten tidak ditemukan")).toBeDefined();
  });

  test("kanal Aplikasi menyembunyikan halaman situs dan mengirim site_page null", async () => {
    const buat = vi.fn(() => ok(konten()));
    render(<EditorKonten csrfToken="c" klien={klienUji({ buat })} />);
    expect(screen.getByLabelText("Halaman situs tujuan")).toBeDefined();
    fireEvent.change(screen.getByLabelText("Kanal"), { target: { value: "app" } });
    expect(screen.queryByLabelText("Halaman situs tujuan")).toBeNull();
    fireEvent.change(screen.getByLabelText("Judul"), { target: { value: "Banner" } });
    fireEvent.click(screen.getByRole("button", { name: "Simpan draf" }));
    await waitFor(() => { expect(buat).toHaveBeenCalledWith(expect.objectContaining({ channel: "app", site_page: null })); });
  });
});
