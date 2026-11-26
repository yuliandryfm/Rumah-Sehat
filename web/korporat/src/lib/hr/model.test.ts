import { describe, expect, test } from "vitest";
import type { AnggotaHr } from "./klien.ts";
import { LABEL_STATUS, dapatDinonaktifkan, ringkas } from "./model.ts";

const hr = (o: Partial<AnggotaHr>): AnggotaHr => ({
  corporate_member_id: "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b",
  email: "hr@perusahaan.example",
  full_name: "Sari",
  job_title: null,
  status: "active",
  is_self: false,
  ...o,
});

describe("status undangan diproses (F1-T107 undangan, ADR-030)", () => {
  test("tampil sebagai teks 'Undangan diproses', dihitung menunggu, tanpa tombol nonaktifkan", () => {
    expect(LABEL_STATUS.invitation_processing.label).toBe("Undangan diproses");
    const daftar = [hr({}), hr({ status: "invitation_processing" }), hr({ status: "pending_activation" })];
    expect(ringkas(daftar)).toEqual({ aktif: 1, menunggu: 2 });
    expect(dapatDinonaktifkan(hr({ status: "invitation_processing" }))).toBe(false);
    expect(dapatDinonaktifkan(hr({ status: "pending_activation" }))).toBe(true);
  });
});
