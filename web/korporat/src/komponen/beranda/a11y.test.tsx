import { cleanup, render, screen } from "@testing-library/react";
import axe from "axe-core";
import { afterEach, beforeAll, describe, expect, test, vi } from "vitest";
import { galat } from "../../lib/hr/klien.ts";
import type { KlienVerifikasi, Muatan, Ringkasan } from "../../lib/verifikasi/klien.ts";
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

const dok = (a: boolean, b: boolean, c: boolean): Ringkasan["documents"] => [
  { kind: "nib", uploaded: a, uploaded_at: a ? "2027-03-01T01:00:00Z" : null },
  { kind: "npwp", uploaded: b, uploaded_at: b ? "2027-03-01T01:00:00Z" : null },
  { kind: "authority_proof", uploaded: c, uploaded_at: c ? "2027-03-01T01:00:00Z" : null },
];
const ringkasan = (o: Partial<Ringkasan>): Ringkasan => ({
  status: "pending_verification",
  submission_round: 1,
  complete: false,
  rejection_reason: null,
  company: { legal_name: "PT Contoh Sintetis", nib: "1234567890123", npwp: "0123456789012345", office_address: "Jl. Contoh No. 1", office_phone: "021-0000000", submitted_at: "2027-03-01T01:15:00Z" },
  documents: dok(false, false, false),
  ...o,
});

const tak = () => Promise.reject(new Error("tidak dipakai"));
const klien = (muat: KlienVerifikasi["muat"]): KlienVerifikasi => ({ muat, unggah: tak, urlUnduh: tak, ajukanUlang: tak, koreksiDanAjukanUlang: tak });
const ok = (m: Muatan) => () => Promise.resolve({ ok: true as const, data: m });
const gagal = (status: number) => () => Promise.resolve(galat(status, {}));

describe("Beranda dalam kerangka member area: aksesibilitas", () => {
  test.each([
    ["aktif", ok({ jenis: "aktif" }), "heading"],
    ["Ditolak", ok({ jenis: "dokumen", ringkasan: ringkasan({ status: "rejected", complete: true, rejection_reason: "nib_document_invalid", documents: dok(true, true, true) }) }), "heading"],
    ["dokumen belum lengkap", ok({ jenis: "dokumen", ringkasan: ringkasan({ documents: dok(true, false, false) }) }), "heading"],
    ["galat 503", gagal(503), "alert"],
    ["galat 403", gagal(403), "alert"],
  ] as const)("%s: tanpa pelanggaran; item menu 'segera hadir' bukan tautan dan tidak fokusable", async (_n, muat, tunggu) => {
    const { container } = render(
      <KerangkaKorporat csrfToken="t">
        <Beranda csrfToken="t" klien={klien(muat)} />
      </KerangkaKorporat>,
    );
    if (tunggu === "heading") await screen.findByRole("heading", { level: 2 });
    else await screen.findByRole("alert");
    expect(await pelanggaranA11y(container)).toEqual([]);
    const nonaktif = container.querySelectorAll("nav .kp-nonaktif");
    expect(nonaktif).toHaveLength(3);
    for (const el of nonaktif) {
      expect(el.querySelector("a")).toBeNull();
      expect(el.hasAttribute("tabindex")).toBe(false);
      expect(el.textContent).toContain("segera hadir");
    }
  });
});
