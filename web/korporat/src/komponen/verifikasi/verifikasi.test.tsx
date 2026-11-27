import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import * as halaman from "../../app/(terautentikasi)/verifikasi/page.tsx";
import { galat as galatHr } from "../../lib/hr/klien.ts";
import { galat, gagalTerhubung, type KlienVerifikasi, type Ringkasan } from "../../lib/verifikasi/klien.ts";
import { StatusVerifikasi } from "./StatusVerifikasi.tsx";

vi.mock("next/navigation", () => ({ usePathname: () => "/verifikasi" }));

const DOK_LENGKAP: Ringkasan["documents"] = [
  { kind: "nib", uploaded: true, uploaded_at: "2027-03-01T01:00:00Z" },
  { kind: "npwp", uploaded: true, uploaded_at: "2027-03-01T01:00:00Z" },
  { kind: "authority_proof", uploaded: true, uploaded_at: "2027-03-01T01:00:00Z" },
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
  documents: [
    { kind: "nib", uploaded: false, uploaded_at: null },
    { kind: "npwp", uploaded: false, uploaded_at: null },
    { kind: "authority_proof", uploaded: false, uploaded_at: null },
  ],
  ...o,
});
const DITOLAK = ringkasan({ status: "rejected", complete: true, rejection_reason: "authority_proof_insufficient", documents: DOK_LENGKAP });

const ok = <T,>(data: T) => Promise.resolve({ ok: true as const, data });
const gagal = (status: number, kode?: string) => Promise.resolve(galat(status, kode === undefined ? {} : { response_code: kode }));

function klien(o: Partial<KlienVerifikasi> = {}, r: Ringkasan = ringkasan()): KlienVerifikasi {
  return {
    muat: () => ok({ jenis: "dokumen" as const, ringkasan: r }),
    unggah: () => ok(r),
    urlUnduh: () => ok({ url: "https://unduh.contoh.test/o?sig=x", expires_at: "2027-03-01T01:20:00Z", mime_type: "image/jpeg", size_bytes: 10 }),
    ajukanUlang: () => ok(ringkasan({ status: "pending_verification", complete: true, documents: DOK_LENGKAP, submission_round: 2 })),
    ...o,
  };
}

const jpg = (ukuran = 4) => new File([new Uint8Array(ukuran)], "nib.jpg", { type: "image/jpeg" });
const grupDokumen = (label: string) => screen.getByRole("group", { name: label });
function pilihBerkas(label: string, berkas: File) {
  const isian = within(grupDokumen(label)).getByLabelText(new RegExp(`^Berkas (pengganti )?${label}$`));
  fireEvent.change(isian, { target: { files: [berkas] } });
  return isian as HTMLInputElement;
}

afterEach(() => {
  cleanup();
});

describe("StatusVerifikasi: Menunggu verifikasi (KR-09, US-1901 AC6)", () => {
  test("status, waktu pengajuan WIB, profil baca-saja dengan NIB/NPWP termasking, tiga dokumen", async () => {
    render(<StatusVerifikasi csrfToken="t" klien={klien()} />);
    expect(await screen.findByText("Menunggu verifikasi")).toBeDefined();
    expect(screen.getByText(/Diajukan 1 Mar 2027.*08\.15 WIB/)).toBeDefined();
    expect(screen.getByText("Lengkapi dokumen perusahaan")).toBeDefined();
    expect(screen.getByText("************0123")).toBeDefined();
    expect(screen.getByText("************2345")).toBeDefined();
    expect(document.body.textContent).not.toContain("1234567890123");
    expect(document.body.textContent).not.toContain("0123456789012345");
    for (const l of ["Dokumen NIB", "Dokumen NPWP", "Bukti kewenangan pendaftar"]) {
      expect(within(grupDokumen(l)).getByText("Belum diunggah")).toBeDefined();
    }
    // Tidak ada formulir koreksi data perusahaan (Q20 belum diputuskan) dan tidak ada Ajukan ulang saat menunggu.
    expect(screen.queryByRole("textbox")).toBeNull();
    expect(screen.queryByRole("button", { name: "Ajukan ulang" })).toBeNull();
    expect(screen.getByText(/PDF bertanda tangan digital tidak diterima/)).toBeDefined();
  });

  test("dokumen lengkap → 'Dokumen sedang diperiksa', tahap pemeriksaan ditandai langkah aktif", async () => {
    render(<StatusVerifikasi csrfToken="t" klien={klien({}, ringkasan({ complete: true, documents: DOK_LENGKAP }))} />);
    expect(await screen.findByText("Dokumen sedang diperiksa")).toBeDefined();
    const aktif = screen.getByRole("list", { name: "Tahapan verifikasi" }).querySelector('[aria-current="step"]');
    expect(aktif?.textContent).toBe("Pemeriksaan dokumen oleh Verifikator");
  });

  test("halaman /verifikasi memakai komponen ini (server component dengan penjaga sesi)", () => {
    expect(typeof halaman.default).toBe("function");
    expect(halaman.metadata).toEqual({ title: "Status verifikasi" });
  });
});

describe("StatusVerifikasi: unggah dan ganti dokumen (US-1901 AC2)", () => {
  test("unggah berhasil: dipanggil dengan jenis dan berkas, status diperbarui, pesan diumumkan tanpa memindah fokus", async () => {
    const unggah = vi.fn<KlienVerifikasi["unggah"]>(() => ok(ringkasan({ documents: [{ kind: "nib", uploaded: true, uploaded_at: "2027-03-01T02:00:00Z" }, ...ringkasan().documents.slice(1)] })));
    render(<StatusVerifikasi csrfToken="t" klien={klien({ unggah })} />);
    await screen.findByText("Menunggu verifikasi");
    const berkas = jpg();
    pilihBerkas("Dokumen NIB", berkas);
    expect(within(grupDokumen("Dokumen NIB")).getByText(/Dipilih: nib\.jpg/)).toBeDefined();
    fireEvent.click(within(grupDokumen("Dokumen NIB")).getByRole("button", { name: "Unggah" }));
    await waitFor(() => {
      expect(within(grupDokumen("Dokumen NIB")).getByText("Terunggah")).toBeDefined();
    });
    expect(unggah).toHaveBeenCalledWith("nib", berkas);
    expect(screen.getByText("Dokumen NIB berhasil diunggah.").getAttribute("role")).toBe("status");
    expect(within(grupDokumen("Dokumen NIB")).getByRole("button", { name: "Ganti dokumen" })).toBeDefined();
  });

  test("negatif: tanpa berkas atau berkas melebihi batas → galat di field (aria-invalid, terhubung), fokus ke isian, server tidak dipanggil", async () => {
    const unggah = vi.fn<KlienVerifikasi["unggah"]>();
    render(<StatusVerifikasi csrfToken="t" klien={klien({ unggah })} />);
    await screen.findByText("Menunggu verifikasi");
    const g = grupDokumen("Dokumen NPWP");
    fireEvent.click(within(g).getByRole("button", { name: "Unggah" }));
    const isian = within(g).getByLabelText("Berkas Dokumen NPWP");
    expect(isian.getAttribute("aria-invalid")).toBe("true");
    expect(document.activeElement).toBe(isian);
    const galatEl = within(g).getByText("Pilih berkas terlebih dahulu.").parentElement;
    expect(isian.getAttribute("aria-describedby")).toContain(galatEl?.id ?? "tidak-ada");

    pilihBerkas("Dokumen NPWP", jpg(5_242_881));
    fireEvent.click(within(g).getByRole("button", { name: "Unggah" }));
    expect(within(g).getByText(/Foto JPG atau PNG paling besar 5 MiB/)).toBeDefined();
    expect(unggah).not.toHaveBeenCalled();
  });

  test("negatif: PDF ditolak server 415 → petunjuk unggah pindaian atau foto di field itu", async () => {
    render(<StatusVerifikasi csrfToken="t" klien={klien({ unggah: () => gagal(415) })} />);
    await screen.findByText("Menunggu verifikasi");
    pilihBerkas("Bukti kewenangan pendaftar", new File([new Uint8Array(4)], "kuasa.pdf", { type: "application/pdf" }));
    fireEvent.click(within(grupDokumen("Bukti kewenangan pendaftar")).getByRole("button", { name: "Unggah" }));
    expect(await within(grupDokumen("Bukti kewenangan pendaftar")).findByText(/Unggah hasil pindaian atau foto dokumen/)).toBeDefined();
    expect(within(grupDokumen("Bukti kewenangan pendaftar")).getByText("Belum diunggah")).toBeDefined();
  });

  test("negatif: gagal terhubung → pesan 'Gagal terhubung' dan berkas pilihan tetap ada", async () => {
    render(<StatusVerifikasi csrfToken="t" klien={klien({ unggah: () => Promise.resolve(gagalTerhubung()) })} />);
    await screen.findByText("Menunggu verifikasi");
    pilihBerkas("Dokumen NIB", jpg());
    fireEvent.click(within(grupDokumen("Dokumen NIB")).getByRole("button", { name: "Unggah" }));
    expect(await within(grupDokumen("Dokumen NIB")).findByText(/Gagal terhubung/)).toBeDefined();
    expect(within(grupDokumen("Dokumen NIB")).getByText(/Dipilih: nib\.jpg/)).toBeDefined();
  });

  test("negatif: storage menolak (status 0) → pesan di field, tanpa pengalihan ke login dan tanpa layar akses", async () => {
    const arahkan = vi.fn();
    const unggah = () => Promise.resolve({ ok: false as const, status: 0, kode: null, pesan: "Berkas gagal diunggah. Pilih berkas lalu unggah lagi." });
    render(<StatusVerifikasi csrfToken="t" klien={klien({ unggah })} arahkan={arahkan} />);
    await screen.findByText("Menunggu verifikasi");
    pilihBerkas("Dokumen NIB", jpg());
    fireEvent.click(within(grupDokumen("Dokumen NIB")).getByRole("button", { name: "Unggah" }));
    expect(await within(grupDokumen("Dokumen NIB")).findByText(/Berkas gagal diunggah/)).toBeDefined();
    expect(arahkan).not.toHaveBeenCalled();
    expect(screen.queryByRole("heading", { level: 1, name: "Menu ini belum dapat dipakai" })).toBeNull();
    expect(screen.getByRole("heading", { level: 1, name: "Status verifikasi perusahaan" })).toBeDefined();
  });

  test("tombol terkunci dengan 'Mengunggah...' selama unggahan (KU-05)", async () => {
    let selesai: (v: Awaited<ReturnType<KlienVerifikasi["unggah"]>>) => void = () => undefined;
    const unggah = vi.fn<KlienVerifikasi["unggah"]>(() => new Promise((r) => { selesai = r; }));
    render(<StatusVerifikasi csrfToken="t" klien={klien({ unggah })} />);
    await screen.findByText("Menunggu verifikasi");
    pilihBerkas("Dokumen NIB", jpg());
    fireEvent.click(within(grupDokumen("Dokumen NIB")).getByRole("button", { name: "Unggah" }));
    const sibuk = await within(grupDokumen("Dokumen NIB")).findByRole("button", { name: "Mengunggah..." });
    expect(sibuk.getAttribute("aria-disabled")).toBe("true");
    fireEvent.click(sibuk);
    expect(unggah).toHaveBeenCalledTimes(1);
    selesai({ ok: true, data: ringkasan() });
    await waitFor(() => {
      expect(within(grupDokumen("Dokumen NIB")).getByRole("button", { name: "Unggah" })).toBeDefined();
    });
  });

  test("unduh: tautan URL berumur pendek tanpa referrer, berlaku sampai waktu WIB", async () => {
    render(<StatusVerifikasi csrfToken="t" klien={klien({}, ringkasan({ documents: DOK_LENGKAP, complete: true }))} />);
    await screen.findByText("Dokumen sedang diperiksa");
    fireEvent.click(within(grupDokumen("Dokumen NIB")).getByRole("button", { name: "Unduh Dokumen NIB" }));
    const tautan = await within(grupDokumen("Dokumen NIB")).findByRole("link", { name: "Buka Dokumen NIB" });
    expect(tautan.getAttribute("href")).toBe("https://unduh.contoh.test/o?sig=x");
    expect(tautan.getAttribute("rel")).toBe("noopener noreferrer");
    expect(within(grupDokumen("Dokumen NIB")).getByText(/berlaku sampai 1 Mar 2027.*08\.20 WIB/)).toBeDefined();
  });
});

describe("StatusVerifikasi: tautan unduh hanya https", () => {
  test.each(["javascript:alert(1)", "http://unduh.contoh.test/o", "data:text/html,x"])(
    "negatif: URL %s tidak dirender sebagai tautan; galat umum",
    async (url) => {
      const urlUnduh = () => ok({ url, expires_at: "2027-03-01T01:20:00Z", mime_type: "image/jpeg", size_bytes: 10 });
      render(<StatusVerifikasi csrfToken="t" klien={klien({ urlUnduh }, ringkasan({ documents: DOK_LENGKAP, complete: true }))} />);
      await screen.findByText("Dokumen sedang diperiksa");
      fireEvent.click(within(grupDokumen("Dokumen NIB")).getByRole("button", { name: "Unduh Dokumen NIB" }));
      expect(await within(grupDokumen("Dokumen NIB")).findByText(/Tautan dokumen tidak dapat dibuka/)).toBeDefined();
      expect(within(grupDokumen("Dokumen NIB")).queryByRole("link")).toBeNull();
      expect(document.querySelector(`a[href="${url}"]`)).toBeNull();
    },
  );
});

describe("StatusVerifikasi: Ditolak dan ajukan ulang (KR-10, US-1902 AC3)", () => {
  test("alasan dari kode tampil sebagai teks; Ajukan ulang → kembali Menunggu verifikasi", async () => {
    const ajukanUlang = vi.fn(() => ok(ringkasan({ complete: true, documents: DOK_LENGKAP, submission_round: 2 })));
    render(<StatusVerifikasi csrfToken="t" klien={klien({ ajukanUlang }, DITOLAK)} />);
    expect(await screen.findByText("Pengajuan ditolak")).toBeDefined();
    expect(screen.getByTestId("alasan-tolak").textContent).toBe("Bukti kewenangan pendaftar tidak memadai");
    expect(document.body.textContent).not.toContain("authority_proof_insufficient");
    fireEvent.click(screen.getByRole("button", { name: "Ajukan ulang" }));
    expect(await screen.findByText(/Pengajuan ulang terkirim/)).toBeDefined();
    expect(ajukanUlang).toHaveBeenCalledTimes(1);
    expect(screen.getByText("Menunggu verifikasi")).toBeDefined();
    expect(screen.queryByRole("button", { name: "Ajukan ulang" })).toBeNull();
  });

  test("negatif: dokumen belum lengkap → pesan tanpa memanggil server", async () => {
    const ajukanUlang = vi.fn(() => ok(DITOLAK));
    const r = ringkasan({ status: "rejected", complete: false, rejection_reason: "nib_document_invalid" });
    render(<StatusVerifikasi csrfToken="t" klien={klien({ ajukanUlang }, r)} />);
    fireEvent.click(await screen.findByRole("button", { name: "Ajukan ulang" }));
    expect(screen.getByText(/Dokumen belum lengkap/)).toBeDefined();
    expect(ajukanUlang).not.toHaveBeenCalled();
  });

  test.each([
    ["4090115", /Dokumen belum lengkap/],
    ["4090001", /sudah menunggu verifikasi/],
    ["4090112", /sudah Aktif/],
    ["4030101", /Muat ulang halaman/],
  ])("negatif: server menolak %s → pesan di dekat tombol, layar tetap", async (kode, pesan) => {
    const status = kode === "4030101" ? 403 : 409;
    render(<StatusVerifikasi csrfToken="t" klien={klien({ ajukanUlang: () => gagal(status, kode) }, DITOLAK)} />);
    fireEvent.click(await screen.findByRole("button", { name: "Ajukan ulang" }));
    const el = await screen.findByText(pesan);
    expect(el.closest("#galat-ajukan")).not.toBeNull();
    expect(screen.getByRole("button", { name: "Ajukan ulang" }).getAttribute("aria-describedby")).toBe("galat-ajukan");
    expect(screen.getByText("Pengajuan ditolak")).toBeDefined();
  });

  test("negatif: sesi berakhir saat mengajukan (401) → dialihkan ke /auth/login", async () => {
    const arahkan = vi.fn();
    render(<StatusVerifikasi csrfToken="t" klien={klien({ ajukanUlang: () => gagal(401, "4010001") }, DITOLAK)} arahkan={arahkan} />);
    fireEvent.click(await screen.findByRole("button", { name: "Ajukan ulang" }));
    await waitFor(() => {
      expect(arahkan).toHaveBeenCalledWith("/auth/login");
    });
  });
});

describe("StatusVerifikasi: keadaan lain", () => {
  test("perusahaan Aktif (4090112 saat memuat) → badge Aktif dan tautan dasbor, tanpa isian unggah", async () => {
    render(<StatusVerifikasi csrfToken="t" klien={klien({ muat: () => ok({ jenis: "aktif" as const }) })} />);
    expect(await screen.findByText("Perusahaan Anda sudah terverifikasi")).toBeDefined();
    expect(screen.getByText("Aktif")).toBeDefined();
    expect(screen.getByRole("link", { name: "Buka dasbor" }).getAttribute("href")).toBe("/");
    expect(screen.queryByRole("group")).toBeNull();
  });

  test("memuat: kerangka tampil sebelum data", () => {
    render(<StatusVerifikasi csrfToken="t" klien={klien({ muat: () => new Promise(() => undefined) })} />);
    expect(document.querySelector(".rs-skeleton, [aria-busy='true']")).not.toBeNull();
  });

  test("negatif: sesi kedaluwarsa saat memuat (401) → dialihkan ke /auth/login", async () => {
    const arahkan = vi.fn();
    render(<StatusVerifikasi csrfToken="t" klien={klien({ muat: () => Promise.resolve(galatHr(401, {})) })} arahkan={arahkan} />);
    await waitFor(() => {
      expect(arahkan).toHaveBeenCalledWith("/auth/login");
    });
  });

  test("negatif: akun nonaktif (401 4010002) → layar akun nonaktif", async () => {
    render(<StatusVerifikasi csrfToken="t" klien={klien({ muat: () => gagal(401, "4010002") })} />);
    expect(await screen.findByRole("heading", { level: 1, name: "Akun nonaktif" })).toBeDefined();
  });

  test("negatif: layanan tidak tersedia → pesan dan 'Coba lagi' memuat ulang", async () => {
    let n = 0;
    const muat = vi.fn(() => (++n === 1 ? gagal(503) : ok({ jenis: "dokumen" as const, ringkasan: ringkasan() })));
    render(<StatusVerifikasi csrfToken="t" klien={klien({ muat })} />);
    expect(await screen.findByText("Status verifikasi tidak dapat dimuat")).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Coba lagi" }));
    expect(await screen.findByText("Menunggu verifikasi")).toBeDefined();
    expect(muat).toHaveBeenCalledTimes(2);
  });
});
