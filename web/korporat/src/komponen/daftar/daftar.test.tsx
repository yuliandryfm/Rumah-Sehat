import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import axe from "axe-core";
import { afterEach, beforeAll, describe, expect, test, vi } from "vitest";
import * as halaman from "../../app/(publik)/daftar/page.tsx";
import { galatDaftar, type KlienDaftar, type TeksPersetujuan } from "../../lib/daftar/klien.ts";
import { FormDaftar } from "./FormDaftar.tsx";

beforeAll(() => {
  document.documentElement.lang = "id";
});
afterEach(cleanup);

const VERSI = "01a2008d-7400-7000-8000-000000000b01";
const VERSI_BARU = "01a2008d-7400-7000-8000-000000000c01";
const WAJIB = "01a2008d-7400-7000-8000-000000000b02";
const teks = (versi = VERSI, draf: string | null = "Teks draf, menunggu persetujuan legal."): TeksPersetujuan => ({
  consent_text_version_id: versi,
  consent_type: "corporate_hr_registration",
  version_label: versi === VERSI ? "0.1-draf" : "0.2-draf",
  effective_from: "2026-11-26T00:00:01Z",
  draft_notice: draf,
  sections: [{ code: "purpose", title: "(b) Tujuan pemrosesan", body: "<script>alert(1)</script> teks biasa" }],
  contact: { title: "Kontak", body: "Kontak Ruang Sehat" },
  items: [{ consent_item_id: WAJIB, item_code: "corporate_hr_data_processing", label: "Saya menyetujui pemrosesan data pendaftaran", is_primary: true, is_required: true, sort_order: 1 }],
});
const PESAN_SERAGAM = "Permintaan pendaftaran diterima. Periksa email Anda untuk melanjutkan. Jika Anda membutuhkan bantuan, hubungi Layanan Pelanggan.";

function klien(o: Partial<KlienDaftar> = {}): KlienDaftar {
  return {
    teks: () => Promise.resolve({ ok: true, data: teks() }),
    kirim: () => Promise.resolve({ ok: true, data: { status: "processing", message: PESAN_SERAGAM } }),
    ...o,
  };
}

const isi = (label: string, nilai: string) => {
  fireEvent.change(screen.getByLabelText(label), { target: { value: nilai } });
};
const lanjut = () => {
  fireEvent.click(screen.getByRole("button", { name: "Lanjut" }));
};
function isiLangkah1() {
  isi("Nama badan usaha", "PT Contoh Sintetis");
  isi("NIB", "1234567890123");
  isi("NPWP perusahaan", "0123456789012345");
  isi("Alamat kantor", "Jl. Contoh No. 1");
  isi("Telepon kantor", "021-0000000");
}
function isiLangkah2() {
  isi("Nama lengkap", "Nama HR Sintetis");
  isi("Jabatan", "Manajer SDM");
  isi("Email kerja", "hr@perusahaan.example");
}
async function keLangkah3() {
  isiLangkah1();
  lanjut();
  await screen.findByRole("heading", { level: 2, name: "Langkah 2 dari 3: Data Anda" });
  isiLangkah2();
  lanjut();
  await screen.findByRole("heading", { level: 2, name: "Langkah 3 dari 3: Persetujuan dan kirim" });
}

async function pelanggaranA11y(wadah: HTMLElement): Promise<string[]> {
  const hasil = await axe.run(wadah, {
    runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"] },
    rules: { "color-contrast": { enabled: false } },
  });
  return hasil.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`);
}

describe("FormDaftar: alur bertahap (US-1901 AC1, brief KR-01-KR-04)", () => {
  test("tiga langkah → kirim dengan versi teks dan butir dicentang → layar Periksa email dengan pesan seragam server", async () => {
    const kirim = vi.fn<KlienDaftar["kirim"]>(() => Promise.resolve({ ok: true, data: { status: "processing", message: PESAN_SERAGAM } }));
    render(<FormDaftar klien={klien({ kirim })} />);
    expect(screen.getByRole("list", { name: "Langkah pendaftaran" }).querySelector('[aria-current="step"]')?.textContent).toContain("Data perusahaan");
    await keLangkah3();
    // Ringkasan isian dan teks persetujuan tampil; teks server sebagai teks biasa, bukan markup.
    expect(screen.getByText("PT Contoh Sintetis")).toBeDefined();
    expect(screen.getByText(/<script>alert\(1\)<\/script> teks biasa/)).toBeDefined();
    expect(document.querySelector("script")).toBeNull();
    expect(screen.getByText("Teks draf, menunggu persetujuan legal.")).toBeDefined();
    const kotak = await screen.findByLabelText("Saya menyetujui pemrosesan data pendaftaran (wajib)");
    expect((kotak as HTMLInputElement).checked).toBe(false);
    fireEvent.click(kotak);
    fireEvent.click(screen.getByRole("button", { name: "Kirim pendaftaran" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Periksa email Anda" })).toBeDefined();
    expect(screen.getByText(PESAN_SERAGAM)).toBeDefined();
    expect(kirim).toHaveBeenCalledWith(expect.objectContaining({ hr_email: "hr@perusahaan.example", npwp: "0123456789012345" }), VERSI, [WAJIB]);
    // Email yang diketik tidak ditampilkan di layar hasil (anti-enumerasi, KR-04).
    expect(document.body.textContent).not.toContain("hr@perusahaan.example");
  });

  test("Kembali mempertahankan isian; Ubah dari ringkasan membuka langkah terkait", async () => {
    render(<FormDaftar klien={klien()} />);
    await keLangkah3();
    fireEvent.click(screen.getByRole("button", { name: "Ubah data perusahaan" }));
    expect(await screen.findByRole("heading", { level: 2, name: "Langkah 1 dari 3: Data perusahaan" })).toBeDefined();
    expect(screen.getByLabelText<HTMLInputElement>("NIB").value).toBe("1234567890123");
    lanjut();
    fireEvent.click(await screen.findByRole("button", { name: "Kembali" }));
    expect((await screen.findByLabelText<HTMLInputElement>("Nama badan usaha")).value).toBe("PT Contoh Sintetis");
  });

  test("halaman /daftar publik: tanpa penjaga sesi", () => {
    expect(halaman.metadata).toEqual({ title: "Daftarkan perusahaan" });
    expect(halaman.default.constructor.name).not.toBe("AsyncFunction");
  });
});

describe("FormDaftar: validasi dan galat (US-1901 AC3, AC5, DG §7)", () => {
  test("negatif: langkah 1 kosong → ringkasan galat diumumkan, galat per field (aria-invalid), fokus ke isian pertama", async () => {
    render(<FormDaftar klien={klien()} />);
    lanjut();
    expect(await screen.findByText("Ada 5 isian yang perlu diperbaiki.")).toBeDefined();
    const nama = screen.getByLabelText("Nama badan usaha");
    expect(nama.getAttribute("aria-invalid")).toBe("true");
    await waitFor(() => {
      expect(document.activeElement).toBe(nama);
    });
    expect(screen.getByRole("heading", { level: 2, name: "Langkah 1 dari 3: Data perusahaan" })).toBeDefined();
  });

  test("negatif: NPWP 15 angka → petunjuk tambahkan 0 di depan, tidak lanjut", async () => {
    render(<FormDaftar klien={klien()} />);
    isiLangkah1();
    isi("NPWP perusahaan", "123456789012345");
    lanjut();
    expect(await screen.findByText(/Tambahkan angka 0 di depan sehingga menjadi 16 angka/)).toBeDefined();
    await waitFor(() => {
      expect(document.activeElement).toBe(screen.getByLabelText("NPWP perusahaan"));
    });
  });

  test("negatif: persetujuan wajib tidak dicentang → galat di kotak, server tidak dipanggil", async () => {
    const kirim = vi.fn<KlienDaftar["kirim"]>();
    render(<FormDaftar klien={klien({ kirim })} />);
    await keLangkah3();
    await screen.findByLabelText("Saya menyetujui pemrosesan data pendaftaran (wajib)");
    fireEvent.click(screen.getByRole("button", { name: "Kirim pendaftaran" }));
    expect(await screen.findByText("Centang persetujuan wajib untuk mengirim pendaftaran.")).toBeDefined();
    expect(kirim).not.toHaveBeenCalled();
  });

  test("negatif: versi persetujuan basi (409 4090106) → teks dimuat ulang, centang dikosongkan, kirim ulang memakai versi baru", async () => {
    let n = 0;
    const teksFn = vi.fn(() => Promise.resolve({ ok: true as const, data: ++n === 1 ? teks() : teks(VERSI_BARU, null) }));
    const kirim = vi
      .fn<KlienDaftar["kirim"]>()
      .mockResolvedValueOnce(galatDaftar(409, { response_code: "4090106" }))
      .mockResolvedValueOnce({ ok: true, data: { status: "processing", message: PESAN_SERAGAM } });
    render(<FormDaftar klien={klien({ teks: teksFn, kirim })} />);
    await keLangkah3();
    fireEvent.click(await screen.findByLabelText("Saya menyetujui pemrosesan data pendaftaran (wajib)"));
    fireEvent.click(screen.getByRole("button", { name: "Kirim pendaftaran" }));
    expect(await screen.findByText(/Teks persetujuan telah berubah/)).toBeDefined();
    expect(await screen.findByText("Versi teks: 0.2-draf")).toBeDefined();
    const kotak = screen.getByLabelText<HTMLInputElement>("Saya menyetujui pemrosesan data pendaftaran (wajib)");
    expect(kotak.checked).toBe(false);
    // Isian tetap ada.
    expect(screen.getByText("PT Contoh Sintetis")).toBeDefined();
    fireEvent.click(kotak);
    fireEvent.click(screen.getByRole("button", { name: "Coba lagi" }));
    await screen.findByRole("heading", { level: 1, name: "Periksa email Anda" });
    expect(kirim.mock.calls[1]?.[1]).toBe(VERSI_BARU);
  });

  test("negatif: 422 dari server → kembali ke langkah field dengan pesan server di field itu", async () => {
    const kirim = vi.fn<KlienDaftar["kirim"]>(() =>
      Promise.resolve(galatDaftar(422, { response_code: "4220001", errors: [{ field: "nib", code: "invalid_format", message: "NIB harus 13 angka" }] })),
    );
    render(<FormDaftar klien={klien({ kirim })} />);
    await keLangkah3();
    fireEvent.click(await screen.findByLabelText("Saya menyetujui pemrosesan data pendaftaran (wajib)"));
    fireEvent.click(screen.getByRole("button", { name: "Kirim pendaftaran" }));
    expect(await screen.findByText("NIB harus 13 angka")).toBeDefined();
    expect(screen.getByRole("heading", { level: 2, name: "Langkah 1 dari 3: Data perusahaan" })).toBeDefined();
    expect(screen.getByLabelText("NIB").getAttribute("aria-invalid")).toBe("true");
  });

  test.each([
    [0, /Gagal terhubung/],
    [429, /Terlalu banyak permintaan/],
    [503, /tidak dapat diproses/],
  ])("negatif: kirim gagal (%s) → 'Pendaftaran belum terkirim', isian tetap, tombol Coba lagi", async (status, pesan) => {
    const g = status === 0 ? { ok: false as const, status: 0, kode: null, pesan: "Gagal terhubung. Pendaftaran belum terkirim dan isian Anda masih ada. Periksa koneksi lalu coba lagi.", isian: {} } : galatDaftar(status, {});
    render(<FormDaftar klien={klien({ kirim: () => Promise.resolve(g) })} />);
    await keLangkah3();
    fireEvent.click(await screen.findByLabelText("Saya menyetujui pemrosesan data pendaftaran (wajib)"));
    fireEvent.click(screen.getByRole("button", { name: "Kirim pendaftaran" }));
    expect(await screen.findByText("Pendaftaran belum terkirim")).toBeDefined();
    expect(screen.getByText(pesan)).toBeDefined();
    expect(screen.getByRole("button", { name: "Coba lagi" })).toBeDefined();
    expect(screen.getByText("PT Contoh Sintetis")).toBeDefined();
  });

  test("negatif: teks persetujuan belum tersedia (503) → pesan dan tombol muat ulang; tidak dapat mengirim", async () => {
    const kirim = vi.fn<KlienDaftar["kirim"]>();
    render(<FormDaftar klien={klien({ teks: () => Promise.resolve(galatDaftar(503, {})), kirim })} />);
    await keLangkah3();
    expect(screen.getByText(/teks persetujuan belum tersedia/)).toBeDefined();
    expect(screen.getByRole("button", { name: "Muat ulang teks" })).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Kirim pendaftaran" }));
    expect(kirim).not.toHaveBeenCalled();
  });

  test("tombol terkunci 'Mengirim...' selama pengiriman (KU-05)", async () => {
    const kirim = vi.fn<KlienDaftar["kirim"]>(() => new Promise(() => undefined));
    render(<FormDaftar klien={klien({ kirim })} />);
    await keLangkah3();
    fireEvent.click(await screen.findByLabelText("Saya menyetujui pemrosesan data pendaftaran (wajib)"));
    fireEvent.click(screen.getByRole("button", { name: "Kirim pendaftaran" }));
    const b = await screen.findByRole("button", { name: "Mengirim..." });
    expect(b.getAttribute("aria-disabled")).toBe("true");
    fireEvent.click(b);
    expect(kirim).toHaveBeenCalledTimes(1);
  });
});

describe("FormDaftar: aksesibilitas otomatis (axe-core)", () => {
  test("langkah 1 dengan galat, langkah 3, dan layar hasil: tanpa pelanggaran", async () => {
    const { container } = render(<halaman.default />);
    expect(await pelanggaranA11y(container)).toEqual([]);
    cleanup();
    const r = render(
      <main>
        <FormDaftar klien={klien()} />
      </main>,
    );
    lanjut();
    await screen.findByText("Ada 5 isian yang perlu diperbaiki.");
    expect(await pelanggaranA11y(r.container)).toEqual([]);
    isiLangkah1();
    lanjut();
    await screen.findByRole("heading", { level: 2, name: "Langkah 2 dari 3: Data Anda" });
    isiLangkah2();
    lanjut();
    fireEvent.click(await screen.findByLabelText("Saya menyetujui pemrosesan data pendaftaran (wajib)"));
    expect(await pelanggaranA11y(r.container)).toEqual([]);
    fireEvent.click(screen.getByRole("button", { name: "Kirim pendaftaran" }));
    await screen.findByRole("heading", { level: 1, name: "Periksa email Anda" });
    expect(await pelanggaranA11y(r.container)).toEqual([]);
  });
});
