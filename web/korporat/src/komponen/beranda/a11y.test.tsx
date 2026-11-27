import { cleanup, render, screen } from "@testing-library/react";
import axe from "axe-core";
import { afterEach, beforeAll, describe, expect, test, vi } from "vitest";
import type { KlienVerifikasi, Muatan } from "../../lib/verifikasi/klien.ts";
import { KerangkaKorporat } from "../KerangkaKorporat.tsx";
import { Beranda } from "./Beranda.tsx";

vi.mock("next/navigation", () => ({ usePathname: () => "/" }));

beforeAll(() => {
  document.documentElement.lang = "id";
});
afterEach(cleanup);

/** Pola `verifikasi/a11y.test.tsx`: axe WCAG 2.x A/AA; kontras diperiksa `pnpm kontras` (jsdom tanpa gaya). */
async function pelanggaranA11y(wadah: HTMLElement): Promise<string[]> {
  const hasil = await axe.run(wadah, {
    runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"] },
    rules: { "color-contrast": { enabled: false } },
  });
  return hasil.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`);
}

const tak = () => Promise.reject(new Error("tidak dipakai"));
const klien = (m: Muatan): KlienVerifikasi =>
  ({ muat: () => Promise.resolve({ ok: true as const, data: m }), unggah: tak, urlUnduh: tak, ajukanUlang: tak, koreksiDanAjukanUlang: tak });

describe("Beranda dalam kerangka member area: aksesibilitas", () => {
  test("aktif: tanpa pelanggaran; menu 'segera hadir' tidak dapat difokuskan", async () => {
    const { container } = render(
      <KerangkaKorporat csrfToken="t">
        <Beranda csrfToken="t" klien={klien({ jenis: "aktif" })} />
      </KerangkaKorporat>,
    );
    await screen.findByRole("heading", { level: 2 });
    expect(await pelanggaranA11y(container)).toEqual([]);
    expect(container.querySelectorAll("nav [aria-disabled=true]")).toHaveLength(3);
    expect(container.querySelectorAll("nav [aria-disabled=true] a, nav [aria-disabled=true][tabindex]")).toHaveLength(0);
  });
});
