import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import axe from "axe-core";
import { afterEach, beforeAll, beforeEach, describe, expect, test, vi } from "vitest";
import type { KlienPerusahaan } from "../../lib/perusahaan/klien.ts";
import { nilaiSebelumnya, type DetailPerusahaan, type FieldDikoreksi } from "../../lib/perusahaan/model.ts";
import { PanelPerusahaan } from "./PanelPerusahaan.tsx";

/**
 * CR-040 (keputusan user Q34 = A): Verifikator melihat penanda "Data dikoreksi" dan nilai sebelumnya di samping
 * setiap field yang dikoreksi HR pada pengajuan ulang. Data sintetis saja; NIB/NPWP sebelumnya datang termasking
 * dari server.
 */

const ID = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
const NIB = "9900000000099";
const NPWP = "9900000000000077";
const NIB_LAMA_TERMASKING = "************0011";
const NPWP_LAMA_TERMASKING = "************0022";

const detail = (corrected_fields: FieldDikoreksi[], o: Partial<DetailPerusahaan> = {}): DetailPerusahaan => ({
  corporate_client_id: ID,
  legal_name: "PT Sintetis Koreksi",
  nib: NIB,
  npwp: NPWP,
  office_address: "Jl. Contoh Baru 2, Jakarta",
  office_phone: "02100000002",
  status: "pending_verification",
  origin: "self_registered",
  submission_round: 2,
  submitted_at: "2027-03-01T01:01:00Z",
  documents: ["nib", "npwp", "authority_proof"],
  documents_version: "a".repeat(64),
  can_decide: true,
  nib_used_by_other_company: false,
  corrected_fields,
  ...o,
});

const SEMUA: FieldDikoreksi[] = [
  { field: "legal_name", previous_value: "PT Sintetis Lama" },
  { field: "nib", previous_value: NIB_LAMA_TERMASKING },
  { field: "npwp", previous_value: NPWP_LAMA_TERMASKING },
  { field: "office_address", previous_value: "Jl. Contoh Lama 1, Jakarta" },
  { field: "office_phone", previous_value: "02100000001" },
];

const ok = <T,>(data: T) => Promise.resolve({ ok: true as const, data });

function klien(d: DetailPerusahaan): KlienPerusahaan {
  return {
    antrean: () =>
      ok({
        items: [
          {
            corporate_client_id: ID,
            legal_name: d.legal_name,
            nib: d.nib,
            origin: "self_registered" as const,
            submission_round: d.submission_round,
            submitted_at: d.submitted_at,
          },
        ],
        hasMore: false,
        nextCursor: null,
      }),
    alasan: () => ok([{ code: "document_invalid", label: "Dokumen tidak valid" }]),
    detail: () => ok(d),
    dokumenUrl: () => ok({ url: "https://neo.example.test/x", expires_at: new Date(Date.now() + 60_000).toISOString(), mime_type: "image/jpeg", size_bytes: 1 }),
    setujui: () => ok({ corporate_client_id: ID, status: "active" as const, submission_round: 2 }),
    tolak: () => ok({ corporate_client_id: ID, status: "rejected" as const, submission_round: 2 }),
    buat: () => ok({ corporate_client_id: ID, status: "active", corporate_member_id: ID, hr_account_id: ID, hr_invitation: "processing" }),
  };
}

async function bukaPanel(d: DetailPerusahaan): Promise<HTMLElement> {
  render(<PanelPerusahaan csrfToken="csrf" klien={klien(d)} />);
  fireEvent.click(await screen.findByRole("button", { name: /Buka\s*pengajuan/ }));
  await screen.findByTestId("npwp-lengkap");
  return screen.getByRole("region", { name: "Detail perusahaan" });
}

/** axe-core, tag WCAG 2.x A/AA + praktik terbaik (pola `web/korporat`); kontras diperiksa `pnpm kontras`. */
async function pelanggaranA11y(wadah: HTMLElement): Promise<string[]> {
  const hasil = await axe.run(wadah, {
    runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"] },
    rules: { "color-contrast": { enabled: false } },
  });
  return hasil.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`);
}

const konsol = { log: vi.spyOn(console, "log"), warn: vi.spyOn(console, "warn"), error: vi.spyOn(console, "error"), info: vi.spyOn(console, "info") };
beforeAll(() => {
  document.documentElement.lang = "id";
});
beforeEach(() => {
  for (const s of Object.values(konsol)) s.mockImplementation(() => undefined);
});
afterEach(() => {
  cleanup();
  for (const s of Object.values(konsol)) s.mockReset();
});

describe("CR-040 penanda Data dikoreksi", () => {
  test("setiap field yang dikoreksi bertanda dan menampilkan nilai sebelumnya di sampingnya", async () => {
    const panel = await bukaPanel(detail(SEMUA));
    const harap: Record<string, string> = {
      legal_name: "PT Sintetis Lama",
      nib: NIB_LAMA_TERMASKING,
      npwp: NPWP_LAMA_TERMASKING,
      office_address: "Jl. Contoh Lama 1, Jakarta",
      office_phone: "02100000001",
    };
    for (const [field, lama] of Object.entries(harap)) {
      const p = within(panel).getByTestId(`koreksi-${field}`);
      expect(p.textContent).toContain("Data dikoreksi");
      expect(p.textContent).toContain(`Sebelumnya: ${lama}`);
    }
    // Penanda berada di baris (dd) field yang sama dengan nilai terbaru.
    expect(within(panel).getByTestId("koreksi-office_address").closest("dd")?.textContent).toContain("Jl. Contoh Baru 2");
    expect(within(panel).getByTestId("koreksi-nib").closest("dd")?.textContent).toContain(NIB);
    // Ringkasan singkat menyebut field yang perlu diperiksa ulang.
    expect(within(panel).getByText(/HR mengoreksi data pada pengajuan ulang ini/)).toBeTruthy();
    expect(panel.textContent).toContain("Nama badan usaha, NIB, NPWP, Alamat kantor, Telepon kantor.");
  });

  test("hanya field yang berubah yang bertanda", async () => {
    const panel = await bukaPanel(detail([{ field: "office_phone", previous_value: "02100000001" }]));
    expect(within(panel).getByTestId("koreksi-office_phone")).toBeTruthy();
    for (const f of ["legal_name", "nib", "npwp", "office_address"]) {
      expect(within(panel).queryByTestId(`koreksi-${f}`)).toBeNull();
    }
    expect(within(panel).getAllByText("Data dikoreksi")).toHaveLength(1);
    // Baris nama badan usaha hanya muncul bila nama dikoreksi (nama sudah menjadi judul panel).
    expect(within(panel).queryByText("Nama badan usaha")).toBeNull();
  });

  test("negatif: tanpa putaran sebelumnya atau tanpa field berubah (pengajuan ulang dokumen saja) tidak ada penanda", async () => {
    for (const d of [detail([], { submission_round: 1 }), detail([], { submission_round: 3 })]) {
      const panel = await bukaPanel(d);
      expect(within(panel).queryByText(/Data dikoreksi/)).toBeNull();
      expect(within(panel).queryByText(/Sebelumnya:/)).toBeNull();
      expect(within(panel).queryByText(/HR mengoreksi data/)).toBeNull();
      cleanup();
    }
  });

  test("negatif: NIB/NPWP sebelumnya tampil termasking apa adanya dari server, tanpa nilai lengkap lama; tidak ke console", async () => {
    const panel = await bukaPanel(detail(SEMUA));
    expect(within(panel).getByTestId("koreksi-nib").textContent).toContain(NIB_LAMA_TERMASKING);
    expect(within(panel).getByTestId("koreksi-npwp").textContent).toContain(NPWP_LAMA_TERMASKING);
    expect(within(panel).getByTestId("koreksi-nib").textContent).not.toMatch(/\d{5,}/);
    expect(within(panel).getByTestId("koreksi-npwp").textContent).not.toMatch(/\d{5,}/);
    const semua = Object.values(konsol).flatMap((s) => s.mock.calls.flat().map(String)).join(" ");
    expect(semua).not.toContain("0011");
    expect(semua).not.toContain("PT Sintetis Lama");
  });

  test("menutup panel membuang nilai sebelumnya dari DOM", async () => {
    await bukaPanel(detail(SEMUA));
    fireEvent.click(screen.getByRole("button", { name: "Tutup" }));
    expect(document.body.textContent).not.toContain("PT Sintetis Lama");
    expect(document.body.textContent).not.toContain("Sebelumnya:");
  });

  test("nilaiSebelumnya: undefined untuk field yang tidak dikoreksi", () => {
    const d = detail([{ field: "nib", previous_value: NIB_LAMA_TERMASKING }]);
    expect(nilaiSebelumnya(d, "nib")).toBe(NIB_LAMA_TERMASKING);
    expect(nilaiSebelumnya(d, "npwp")).toBeUndefined();
  });

  test("aksesibilitas (axe): panel detail dengan penanda dan tanpa penanda tanpa pelanggaran", async () => {
    let panel = await bukaPanel(detail(SEMUA));
    expect(await pelanggaranA11y(panel)).toEqual([]);
    cleanup();
    panel = await bukaPanel(detail([]));
    expect(await pelanggaranA11y(panel)).toEqual([]);
  });
});
