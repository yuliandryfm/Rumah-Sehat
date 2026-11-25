import { cleanup, render, screen, within } from "@testing-library/react";
import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, describe, expect, test, vi } from "vitest";
import AkunNonaktif from "../app/(publik)/akun-nonaktif/page.tsx";
import MasukGagal from "../app/(publik)/masuk-gagal/page.tsx";
import Galat from "../app/error.tsx";
import GalatGlobal from "../app/global-error.tsx";
import { DataTidakDitemukan } from "./DataTidakDitemukan.tsx";
import { KerangkaKorporat } from "./KerangkaKorporat.tsx";

const jalur = vi.hoisted(() => ({ nilai: "/" }));
vi.mock("next/navigation", () => ({ usePathname: () => jalur.nilai }));

afterEach(() => {
  cleanup();
  jalur.nilai = "/";
});

describe("KerangkaKorporat", () => {
  test("landmark: tautan lewati, banner, navigasi Menu, main#isi; Keluar = formulir POST dengan csrf_token", () => {
    render(
      <KerangkaKorporat csrfToken="token-csrf-uji">
        <p>isi</p>
      </KerangkaKorporat>,
    );
    expect(screen.getByRole("link", { name: "Lewati ke isi" }).getAttribute("href")).toBe("#isi");
    expect(screen.getByRole("banner")).toBeDefined();
    expect(screen.getByRole("main").id).toBe("isi");
    expect(screen.getByRole("navigation", { name: "Menu" })).toBeDefined();
    const form = screen.getByRole("button", { name: "Keluar" }).closest("form");
    expect(form?.getAttribute("method")).toBe("post");
    expect(form?.getAttribute("action")).toBe("/auth/logout");
    expect(form?.querySelector<HTMLInputElement>("input[name=csrf_token]")?.value).toBe("token-csrf-uji");
  });

  test("halaman aktif ditandai; layanan belum tersedia tampil nonaktif dengan alasan, bukan tautan", () => {
    render(
      <KerangkaKorporat csrfToken="t">
        <p>isi</p>
      </KerangkaKorporat>,
    );
    const nav = screen.getByRole("navigation", { name: "Menu" });
    expect(within(nav).getAllByRole("link").map((a) => a.textContent)).toEqual(["Dasbor", "Kelola HR"]);
    expect(within(nav).getByRole("link", { name: "Dasbor" }).getAttribute("aria-current")).toBe("page");
    for (const nama of ["Pesanan", "Peserta", "Invoice"]) {
      const el = within(nav).getByText(`${nama} (belum tersedia)`);
      expect(el.getAttribute("aria-disabled")).toBe("true");
      expect(el.closest("a")).toBeNull();
    }
  });
});

describe("layar akses ditolak, akun nonaktif, galat", () => {
  test("akses ditolak = 404 (KU-03): tidak membocorkan keberadaan data, ada kode referensi", () => {
    render(<DataTidakDitemukan kode="0193a5e6-7f00-7abc-8def-0123456789ab" />);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Data tidak ditemukan");
    expect(screen.getByText("Kode referensi: 0193a5e6-7f00-7abc-8def-0123456789ab")).toBeDefined();
    expect(screen.getByRole("link", { name: "Kembali ke beranda" }).getAttribute("href")).toBe("/");
    expect(document.body.textContent).not.toMatch(/tidak berhak|izin|forbidden|403|perusahaan lain/i);
  });

  test("akun nonaktif: satu pesan wireframe K01 yang sama dengan masuk gagal, tanpa menyebut nonaktif/terkunci", () => {
    render(<AkunNonaktif />);
    expect(screen.getByRole("alert").textContent).toContain("Email, password, atau kode tidak sesuai");
    expect(document.body.textContent).not.toMatch(/nonaktif|terkunci|tidak ada akun/i);
  });

  test("masuk gagal: pesan umum, ajakan masuk lagi ke /auth/login, tanpa tautan ke akun nonaktif", async () => {
    render(await MasukGagal({ searchParams: Promise.resolve({ auth_error: "failed" }) }));
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Masuk tidak berhasil");
    expect(screen.getByRole("link", { name: "Masuk lagi" }).getAttribute("href")).toBe("/auth/login");
    expect(screen.queryByRole("link", { name: /nonaktif/i })).toBeNull();
  });

  test("masuk gagal karena kedaluwarsa: pesan khusus kedaluwarsa", async () => {
    render(await MasukGagal({ searchParams: Promise.resolve({ auth_error: "expired" }) }));
    expect(screen.getByText("Proses masuk kedaluwarsa. Silakan masuk lagi.")).toBeDefined();
  });

  test("galat server: kode referensi dari digest; pesan galat internal tidak tampil", () => {
    const galat = Object.assign(new Error("connection refused 10.0.0.5 tabel corporate_member"), { digest: "1234567890" });
    render(<Galat error={galat} reset={() => undefined} />);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Terjadi kesalahan");
    expect(screen.getByText("Kode referensi: 1234567890")).toBeDefined();
    expect(screen.getByRole("button", { name: "Coba lagi" })).toBeDefined();
    expect(document.body.textContent).not.toMatch(/10\.0\.0\.5|corporate_member|refused/);
  });

  test("galat tanpa digest: tanpa baris kode referensi, tetap pesan umum", () => {
    render(<Galat error={new Error("rahasia")} reset={() => undefined} />);
    expect(screen.queryByText(/Kode referensi/)).toBeNull();
    expect(document.body.textContent).not.toContain("rahasia");
  });

  test("galat global: kode referensi tampil, pesan galat tidak", () => {
    const galat = Object.assign(new Error("rahasia"), { digest: "abcdef123456" });
    const html = renderToStaticMarkup(<GalatGlobal error={galat} reset={() => undefined} />);
    expect(html).toContain("Kode referensi: abcdef123456");
    expect(html).not.toContain("rahasia");
  });
});
