import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { SessionTimeoutWarning } from "./SessionTimeoutWarning.tsx";

const CSRF = "c".repeat(43);

function status(idleInMs: number, absoluteInMs: number, csrf = CSRF): Response {
  const now = Date.now();
  return Response.json({
    response_code: "2000000",
    response_description: "Berhasil",
    data: {
      idle_expires_at: new Date(now + idleInMs).toISOString(),
      absolute_expires_at: new Date(now + absoluteInMs).toISOString(),
      server_time: new Date(now).toISOString(),
      warning_seconds: 120,
      csrf_token: csrf,
    },
  });
}

async function mount(fetchMock: ReturnType<typeof vi.fn>, onExpired = vi.fn()) {
  vi.stubGlobal("fetch", fetchMock);
  render(<SessionTimeoutWarning onExpired={onExpired} />);
  await act(async () => {
    await Promise.resolve();
  });
  return onExpired;
}

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("SessionTimeoutWarning (trace: KEPUTUSAN-170, NFR-SEC-04)", () => {
  test("sisa tidak aktif > 2 menit → tidak ada dialog", async () => {
    await mount(vi.fn().mockResolvedValue(status(10 * 60_000, 6 * 3600_000)));
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });

  test("sisa ≤ 2 menit → dialog dengan 'Tetap masuk' dan 'Keluar sekarang' (teks wireframe C01)", async () => {
    await mount(vi.fn().mockResolvedValue(status(119_000, 6 * 3600_000)));
    const dlg = screen.getByRole("alertdialog");
    expect(dlg.textContent).toContain("Sesi akan berakhir dalam 2 menit");
    expect(dlg.textContent).toContain("Anda tidak aktif selama hampir 30 menit");
    expect(screen.getByRole("button", { name: "Tetap masuk" })).toBe(document.activeElement);
    expect(screen.getByRole("button", { name: "Keluar sekarang" })).toBeTruthy();
  });

  test("'Tetap masuk' → POST extend dengan header X-CSRF-Token; dialog tertutup setelah diperpanjang", async () => {
    const f = vi.fn().mockResolvedValueOnce(status(90_000, 6 * 3600_000)).mockResolvedValueOnce(status(30 * 60_000, 6 * 3600_000));
    await mount(f);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Tetap masuk" }));
      await Promise.resolve();
    });
    await act(async () => {
      await Promise.resolve();
    });
    const [url, init] = f.mock.calls[1] as [string, RequestInit];
    expect(url).toBe("/auth/session/extend");
    expect(init.method).toBe("POST");
    expect(new Headers(init.headers).get("X-CSRF-Token")).toBe(CSRF);
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });

  test("'Keluar sekarang' adalah formulir POST /auth/logout dengan csrf_token tersembunyi", async () => {
    await mount(vi.fn().mockResolvedValue(status(60_000, 6 * 3600_000)));
    const form = screen.getByRole("button", { name: "Keluar sekarang" }).closest("form");
    expect(form?.getAttribute("method")).toBe("post");
    expect(form?.getAttribute("action")).toBe("/auth/logout");
    expect((form?.querySelector('input[name="csrf_token"]') as HTMLInputElement | null)?.value).toBe(CSRF);
  });

  test("batas mutlak 12 jam → peringatan tanpa perpanjangan", async () => {
    await mount(vi.fn().mockResolvedValue(status(100_000, 100_000)));
    const dlg = screen.getByRole("alertdialog");
    expect(dlg.textContent).toContain("tidak dapat diperpanjang");
    expect(screen.queryByRole("button", { name: "Tetap masuk" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Mengerti" }));
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });

  test("status 401 (sesi berakhir/dicabut) → onExpired", async () => {
    const onExpired = await mount(vi.fn().mockResolvedValue(new Response(null, { status: 401 })));
    expect(onExpired).toHaveBeenCalledTimes(1);
  });

  test("perpanjangan ditolak 401 → onExpired; gagal jaringan → pesan galat, dialog tetap", async () => {
    const f = vi
      .fn()
      .mockResolvedValueOnce(status(60_000, 6 * 3600_000))
      .mockRejectedValueOnce(new TypeError("jaringan"));
    const onExpired = await mount(f);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Tetap masuk" }));
      await Promise.resolve();
    });
    expect(screen.getByRole("alert").textContent).toContain("Sesi gagal diperpanjang");
    expect(screen.getByRole("alertdialog")).toBeTruthy();
    f.mockResolvedValueOnce(new Response(null, { status: 401 }));
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Tetap masuk" }));
      await Promise.resolve();
    });
    expect(onExpired).toHaveBeenCalledTimes(1);
  });

  test("HTML yang dirender tidak memuat token OIDC (hanya token CSRF untuk formulir keluar)", async () => {
    await mount(vi.fn().mockResolvedValue(status(60_000, 6 * 3600_000)));
    const html = document.body.innerHTML;
    expect(html).not.toMatch(/access_token|refresh_token|id_token|Bearer/);
  });
});
