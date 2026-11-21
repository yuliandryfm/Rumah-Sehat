import { cleanup, render, screen, within } from "@testing-library/react";
import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, describe, expect, test, vi } from "vitest";
import MasukGagal from "../app/(publik)/masuk-gagal/page.tsx";
import AkunNonaktif from "../app/(publik)/akun-nonaktif/page.tsx";
import Galat from "../app/error.tsx";
import GalatGlobal from "../app/global-error.tsx";
import type { ItemNav } from "../lib/navigasi.ts";
import { DataTidakDitemukan } from "./DataTidakDitemukan.tsx";
import { KerangkaBackoffice } from "./KerangkaBackoffice.tsx";

const jalur = vi.hoisted(() => ({ nilai: "/" }));
vi.mock("next/navigation", () => ({ usePathname: () => jalur.nilai }));

afterEach(() => {
  cleanup();
  jalur.nilai = "/";
});

const MENU: readonly ItemNav[] = [
  { href: "/", label: "Beranda", untuk: "semua" },
  { href: "/uji", label: "Menu Verifikator", untuk: ["Verifikator"] },
];

describe("KerangkaBackoffice", () => {
  test("landmark: tautan lewati, banner, navigasi Menu, main#isi; Keluar = formulir POST dengan csrf_token", () => {
    render(
      <KerangkaBackoffice profil={null} csrfToken="token-csrf-uji" daftarMenu={MENU}>
        <p>isi</p>
      </KerangkaBackoffice>,
    );
    expect(screen.getByRole("link", { name: "Lewati ke isi" }).getAttribute("href")).toBe("#isi");
    expect(screen.getByRole("banner")).toBeDefined();
    expect(screen.getByRole("main").id).toBe("isi");
    expect(screen.getByRole("navigation", { name: "Menu" })).toBeDefined();
    const tombol = screen.getByRole("button", { name: "Keluar" });
    const form = tombol.closest("form");
    expect(form?.getAttribute("method")).toBe("post");
    expect(form?.getAttribute("action")).toBe("/auth/logout");
    expect(form?.querySelector<HTMLInputElement>("input[name=csrf_token]")?.value).toBe("token-csrf-uji");
  });

  test("menu mengikuti sub-peran; halaman aktif ditandai aria-current", () => {
    jalur.nilai = "/uji/detail";
    render(
      <KerangkaBackoffice profil={{ email: "admin@contoh.id", subPeran: ["Verifikator"] }} csrfToken="t" daftarMenu={MENU}>
        <p>isi</p>
      </KerangkaBackoffice>,
    );
    const nav = screen.getByRole("navigation", { name: "Menu" });
    expect(within(nav).getAllByRole("link").map((a) => a.textContent)).toEqual(["Beranda", "Menu Verifikator"]);
    expect(within(nav).getByRole("link", { name: "Menu Verifikator" }).getAttribute("aria-current")).toBe("page");
    expect(within(nav).getByRole("link", { name: "Beranda" }).getAttribute("aria-current")).toBeNull();
    expect(screen.getByText(/admin@contoh\.id/)).toBeDefined();
  });

  test("negatif: sub-peran lain tidak melihat menu Verifikator", () => {
    render(
      <KerangkaBackoffice profil={{ email: "k@contoh.id", subPeran: ["Keuangan"] }} csrfToken="t" daftarMenu={MENU}>
        <p>isi</p>
      </KerangkaBackoffice>,
    );
    expect(screen.queryByRole("link", { name: "Menu Verifikator" })).toBeNull();
    expect(screen.getByRole("link", { name: "Beranda" })).toBeDefined();
  });
});

describe("layar akses ditolak, akun nonaktif, galat", () => {
  test("akses ditolak: pesan tidak membocorkan keberadaan data, ada kode referensi dan jalan keluar", () => {
    render(<DataTidakDitemukan kode="0193a5e6-7f00-7abc-8def-0123456789ab" />);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Data tidak ditemukan");
    expect(screen.getByText("Kode referensi: 0193a5e6-7f00-7abc-8def-0123456789ab")).toBeDefined();
    expect(screen.getByRole("link", { name: "Kembali ke beranda" }).getAttribute("href")).toBe("/");
    expect(document.body.textContent).not.toMatch(/tidak berhak|izin|forbidden|403/i);
  });

  test("akun nonaktif (C01, KU-10): peringatan dan arahan menghubungi admin", () => {
    render(<AkunNonaktif />);
    expect(screen.getByRole("status").textContent).toContain("Akun nonaktif");
    expect(screen.getByText("Hubungi admin Ruang Sehat.")).toBeDefined();
  });

  test("masuk gagal: pesan umum, ajakan masuk lagi, tanpa tautan ke akun nonaktif", async () => {
    render(await MasukGagal({ searchParams: Promise.resolve({ auth_error: "failed" }) }));
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Masuk tidak berhasil");
    expect(screen.getByRole("link", { name: "Masuk lagi" }).getAttribute("href")).toBe("/auth/login");
    expect(screen.queryByRole("link", { name: /nonaktif/i })).toBeNull();
  });

  test("galat server: kode referensi dari digest; pesan galat internal tidak tampil", () => {
    const galat = Object.assign(new Error("connection refused 10.0.0.5 tabel patients"), { digest: "1234567890" });
    render(<Galat error={galat} reset={() => undefined} />);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Terjadi kesalahan");
    expect(screen.getByText("Kode referensi: 1234567890")).toBeDefined();
    expect(screen.getByRole("button", { name: "Coba lagi" })).toBeDefined();
    expect(document.body.textContent).not.toMatch(/10\.0\.0\.5|patients|refused/);
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
