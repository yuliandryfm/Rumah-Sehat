import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { KerangkaSitus } from "./KerangkaSitus.tsx";
import { JALUR_LOGO, Logo } from "./Logo.tsx";

const jalur = vi.hoisted(() => ({ nilai: "/" }));
vi.mock("next/navigation", () => ({ usePathname: () => jalur.nilai }));

const sini = dirname(fileURLToPath(import.meta.url));
const akarRepo = join(sini, "../../../..");

afterEach(() => {
  cleanup();
  jalur.nilai = "/";
  vi.unstubAllEnvs();
});

describe("tampilan baru (CR-042): logo", () => {
  test("jalur logo sama persis dengan berkas SVG dari paket desain (docs/design/aset)", () => {
    const svg = readFileSync(join(akarRepo, "docs/design/aset/logo-ruang-sehat.svg"), "utf8");
    const d = [...svg.matchAll(/\sd="([^"]+)"/g)].map((m) => m[1]);
    expect(d).toEqual([JALUR_LOGO.rumah, JALUR_LOGO.daun, JALUR_LOGO.urat]);
  });

  test("logo dekoratif (aria-hidden, tidak dapat difokus); nama merek ada sebagai teks di header dan footer", () => {
    const { container } = render(<Logo />);
    const svg = container.querySelector("svg");
    expect(svg?.getAttribute("aria-hidden")).toBe("true");
    expect(svg?.getAttribute("focusable")).toBe("false");
    cleanup();
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    expect(within(screen.getByRole("banner")).getByRole("link", { name: "Ruang Sehat" }).getAttribute("href")).toBe("/");
    expect(within(screen.getByRole("contentinfo")).getByText("Ruang Sehat")).toBeDefined();
  });
});

describe("tampilan baru (CR-042): pita ajakan unduh", () => {
  test("tampil di akhir isi (di dalam main) dengan dua tautan toko yang jatuh ke /unduh bila env kosong", () => {
    vi.stubEnv("SITUS_PLAYSTORE_URL", "");
    vi.stubEnv("SITUS_APPSTORE_URL", "");
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    const band = within(screen.getByRole("main")).getByRole("region", { name: "Mulai hidup lebih sehat bersama Ruang Sehat" });
    const tautan = within(band).getAllByRole("link");
    expect(tautan.map((a) => a.textContent)).toEqual(["Google Play", "App Store"]);
    expect(tautan.map((a) => a.getAttribute("href"))).toEqual(["/unduh", "/unduh"]);
    // Tanpa klaim harga yang belum diputuskan dan tanpa angka karangan.
    expect(band.textContent).not.toMatch(/gratis|\d/i);
  });

  test("alamat toko valid dipakai juga di pita; tidak tampil di halaman Unduh aplikasi", () => {
    vi.stubEnv("SITUS_PLAYSTORE_URL", "https://play.google.com/store/apps/details?id=id.contoh.uji");
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    const band = screen.getByRole("region", { name: "Mulai hidup lebih sehat bersama Ruang Sehat" });
    expect(within(band).getByRole("link", { name: "Unduh di Google Play" }).getAttribute("rel")).toBe("noopener noreferrer");
    cleanup();
    jalur.nilai = "/unduh";
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    expect(screen.queryByRole("region", { name: "Mulai hidup lebih sehat bersama Ruang Sehat" })).toBeNull();
  });
});

describe("tampilan baru (CR-042): tanpa sumber daya eksternal", () => {
  test("CSS dan kode situs tidak merujuk CDN, Google Fonts, foto googleusercontent, atau ikon Material Symbols", () => {
    const berkas = [
      join(sini, "../app/situs.css"),
      join(sini, "../app/font.ts"),
      join(sini, "Logo.tsx"),
      join(sini, "KerangkaSitus.tsx"),
      join(sini, "BandUnduh.tsx"),
      join(sini, "Beranda.tsx"),
    ];
    for (const b of berkas) {
      const isi = readFileSync(b, "utf8").replace(/\/\*[\s\S]*?\*\//g, "");
      expect(isi, b).not.toMatch(/googleapis|gstatic|googleusercontent|cdn\.tailwindcss|material-symbols|https?:\/\/(?!www\.w3\.org)/i);
    }
  });

  test("tiga keluarga font desain disajikan lokal (next/font/local) beserta lisensi OFL-nya", () => {
    const font = readFileSync(join(sini, "../app/font.ts"), "utf8");
    expect(font.match(/src: "\.\/fonts\/[A-Za-z0-9-]+\.woff2"/g)).toHaveLength(3);
    for (const f of ["OFL-InstrumentSans.txt", "OFL-Alata.txt", "OFL-SplineSansMono.txt"]) {
      expect(readFileSync(join(sini, "../app/fonts", f), "utf8")).toContain("SIL Open Font License, Version 1.1");
    }
  });
});

describe("tampilan baru (CR-042): isi layanan tetap tujuh layanan rilis 1", () => {
  test("kode situs tidak memuat layanan di luar rilis 1 yang ada di mockup (vaksinasi/infus untuk pasien)", () => {
    const isi = readFileSync(join(sini, "Beranda.tsx"), "utf8") + readFileSync(join(sini, "KerangkaSitus.tsx"), "utf8") + readFileSync(join(sini, "BandUnduh.tsx"), "utf8");
    expect(isi).not.toMatch(/Suntik|Infus|Vaksinasi/);
  });
});
