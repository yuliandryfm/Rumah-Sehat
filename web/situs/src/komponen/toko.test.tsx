import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { KerangkaSitus } from "./KerangkaSitus.tsx";
import { TautanToko } from "./TautanToko.tsx";

vi.mock("next/navigation", () => ({ usePathname: () => "/" }));

afterEach(() => {
  cleanup();
  vi.unstubAllEnvs();
});

const PLAY = "https://play.google.com/store/apps/details?id=id.contoh.uji";
const APPLE = "https://apps.apple.com/id/app/contoh/id0000000000";

describe("TautanToko (CR-038)", () => {
  test("env kosong: kedua tautan mengarah ke halaman Unduh aplikasi (internal, tanpa rel eksternal)", () => {
    vi.stubEnv("SITUS_PLAYSTORE_URL", "");
    vi.stubEnv("SITUS_APPSTORE_URL", "");
    render(<TautanToko />);
    const grup = screen.getByRole("group", { name: "Unduh aplikasi" });
    const tautan = within(grup).getAllByRole("link");
    expect(tautan.map((a) => a.getAttribute("href"))).toEqual(["/unduh", "/unduh"]);
    expect(tautan.map((a) => a.textContent)).toEqual(["Google Play", "App Store"]);
    for (const a of tautan) expect(a.getAttribute("rel")).toBeNull();
  });

  test("env valid: tautan toko eksternal dengan rel noopener noreferrer dan nama aksesibel memuat teks tampak", () => {
    vi.stubEnv("SITUS_PLAYSTORE_URL", PLAY);
    vi.stubEnv("SITUS_APPSTORE_URL", APPLE);
    render(<TautanToko />);
    const play = screen.getByRole("link", { name: "Unduh di Google Play" });
    const apple = screen.getByRole("link", { name: "Unduh di App Store" });
    expect(play.getAttribute("href")).toBe(PLAY);
    expect(apple.getAttribute("href")).toBe(APPLE);
    expect(play.getAttribute("rel")).toBe("noopener noreferrer");
    expect(apple.getAttribute("rel")).toBe("noopener noreferrer");
  });

  test("env tidak valid atau tertukar: kembali ke /unduh (fail-closed)", () => {
    vi.stubEnv("SITUS_PLAYSTORE_URL", APPLE);
    vi.stubEnv("SITUS_APPSTORE_URL", "javascript:alert(1)");
    render(<TautanToko />);
    expect(within(screen.getByRole("group")).getAllByRole("link").map((a) => a.getAttribute("href"))).toEqual(["/unduh", "/unduh"]);
  });

  test("tautan toko ada di banner header bersama navigasi", () => {
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    const banner = screen.getByRole("banner");
    expect(within(banner).getByRole("group", { name: "Unduh aplikasi" })).toBeDefined();
    expect(within(banner).getByRole("navigation", { name: "Utama" })).toBeDefined();
  });
});
