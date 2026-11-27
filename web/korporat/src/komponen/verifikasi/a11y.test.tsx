import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import axe from "axe-core";
import { afterEach, beforeAll, describe, expect, test, vi } from "vitest";
import { KerangkaKorporat } from "../KerangkaKorporat.tsx";
import type { KlienVerifikasi, Ringkasan } from "../../lib/verifikasi/klien.ts";
import { StatusVerifikasi } from "./StatusVerifikasi.tsx";

vi.mock("next/navigation", () => ({ usePathname: () => "/verifikasi" }));

beforeAll(() => {
  document.documentElement.lang = "id";
});
afterEach(cleanup);

/**
 * Pemeriksaan aksesibilitas otomatis layar status verifikasi (DG §7.2, §7.4): axe-core dengan tag WCAG 2.x A/AA
 * dan praktik terbaik, pola `web/situs/src/komponen/a11y.test.tsx`. `color-contrast` dimatikan karena jsdom tidak
 * menghitung gaya; kontras token diperiksa `pnpm kontras`.
 */
async function pelanggaranA11y(wadah: HTMLElement): Promise<string[]> {
  const hasil = await axe.run(wadah, {
    runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"] },
    rules: { "color-contrast": { enabled: false } },
  });
  return hasil.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`);
}

const DOK: Ringkasan["documents"] = [
  { kind: "nib", uploaded: true, uploaded_at: "2027-03-01T01:00:00Z" },
  { kind: "npwp", uploaded: false, uploaded_at: null },
  { kind: "authority_proof", uploaded: true, uploaded_at: "2027-03-01T01:00:00Z" },
];
const R: Ringkasan = {
  status: "rejected",
  submission_round: 1,
  complete: false,
  rejection_reason: "nib_document_invalid",
  company: {
    legal_name: "PT Contoh Sintetis",
    nib: "1234567890123",
    npwp: "0123456789012345",
    office_address: "Jl. Contoh No. 1",
    office_phone: "021-0000000",
    submitted_at: "2027-03-01T01:15:00Z",
  },
  documents: DOK,
};
const klien = (r: Ringkasan): KlienVerifikasi => ({
  muat: () => Promise.resolve({ ok: true, data: { jenis: "dokumen", ringkasan: r } }),
  unggah: () => Promise.resolve({ ok: true, data: r }),
  urlUnduh: () => Promise.resolve({ ok: false, status: 404, kode: null, pesan: "x" }),
  ajukanUlang: () => Promise.resolve({ ok: true, data: r }),
  koreksiDanAjukanUlang: () => Promise.resolve({ ok: true, data: r }),
});

describe("aksesibilitas otomatis status verifikasi (axe-core)", () => {
  test.each<[string, Ringkasan]>([
    ["Ditolak, dokumen belum lengkap", R],
    ["Menunggu verifikasi, dokumen lengkap", { ...R, status: "pending_verification", complete: true, rejection_reason: null, documents: DOK.map((d) => ({ ...d, uploaded: true })) }],
  ])("%s dalam kerangka member area: tanpa pelanggaran", async (_nama, r) => {
    const { container } = render(
      <KerangkaKorporat csrfToken="t">
        <StatusVerifikasi csrfToken="t" klien={klien(r)} />
      </KerangkaKorporat>,
    );
    await screen.findByRole("heading", { level: 2, name: "Profil perusahaan" });
    expect(await pelanggaranA11y(container)).toEqual([]);
  });

  test("galat per field dan galat ajukan ulang: tetap tanpa pelanggaran", async () => {
    const { container } = render(
      <KerangkaKorporat csrfToken="t">
        <StatusVerifikasi csrfToken="t" klien={klien(R)} />
      </KerangkaKorporat>,
    );
    await screen.findByText("Pengajuan ditolak");
    fireEvent.click(within(screen.getByRole("group", { name: "Dokumen NPWP" })).getByRole("button", { name: "Unggah" }));
    fireEvent.click(screen.getByRole("button", { name: "Ajukan ulang" }));
    expect(await pelanggaranA11y(container)).toEqual([]);
  });

  test("formulir koreksi data (CR-036) dengan galat per field: tetap tanpa pelanggaran", async () => {
    const lengkap: Ringkasan = { ...R, complete: true, documents: DOK.map((d) => ({ ...d, uploaded: true })) };
    const { container } = render(
      <KerangkaKorporat csrfToken="t">
        <StatusVerifikasi csrfToken="t" klien={klien(lengkap)} />
      </KerangkaKorporat>,
    );
    await screen.findByText("Pengajuan ditolak");
    fireEvent.click(screen.getByLabelText("Ubah NPWP"));
    fireEvent.change(screen.getByLabelText("NPWP baru"), { target: { value: "123" } });
    fireEvent.change(screen.getByLabelText("Telepon kantor"), { target: { value: "" } });
    fireEvent.click(screen.getByRole("button", { name: "Ajukan ulang" }));
    await screen.findByText("Telepon kantor wajib diisi.");
    expect(await pelanggaranA11y(container)).toEqual([]);
  });

  test("urutan judul logis: h1 lalu h2 bagian, h3 per dokumen", async () => {
    render(<StatusVerifikasi csrfToken="t" klien={klien(R)} />);
    await screen.findByText("Pengajuan ditolak");
    const judul = [...document.querySelectorAll("h1, h2, h3")].map((h) => `${h.tagName}:${h.textContent}`);
    expect(judul).toEqual([
      "H1:Status verifikasi perusahaan",
      "H2:Dokumen perusahaan",
      "H3:Dokumen NIB",
      "H3:Dokumen NPWP",
      "H3:Bukti kewenangan pendaftar",
      "H2:Ajukan ulang",
      "H2:Profil perusahaan",
    ]);
  });
});
