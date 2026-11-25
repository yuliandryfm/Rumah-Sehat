import { Alert } from "@rs/ui";
import type { Metadata } from "next";
import { LayarStatus } from "../../../komponen/LayarStatus.tsx";

export const metadata: Metadata = { title: "Tidak dapat masuk" };

/**
 * Layar akun nonaktif. K01 layar 1 menetapkan SATU pesan untuk password salah, akun tidak ada, terkunci,
 * nonaktif, dan kode salah (US-103 AC2, UP-73): teks di bawah dari wireframe itu, tanpa menyebut "nonaktif"
 * supaya status akun tidak bocor. Layar akun nonaktif untuk pemegang sesi yang sah (401 4010002, KU-10) ada di
 * `komponen/hr/GalatAkses.tsx` (F1-T108), bukan di rute publik ini.
 */
export default function AkunNonaktif() {
  return (
    <LayarStatus judul="Tidak dapat masuk">
      <Alert variant="danger" title="Tidak dapat masuk">
        Email, password, atau kode tidak sesuai. Bila tidak dapat masuk, gunakan Lupa password atau hubungi Layanan Pelanggan.
      </Alert>
    </LayarStatus>
  );
}
