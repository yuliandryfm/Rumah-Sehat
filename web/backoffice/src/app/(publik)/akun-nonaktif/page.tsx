import { Alert } from "@rs/ui";
import type { Metadata } from "next";
import { LayarStatus } from "../../../komponen/LayarStatus.tsx";

export const metadata: Metadata = { title: "Akun nonaktif" };

/**
 * Layar akun nonaktif. Teks persis dari wireframe C01 layar 1 ("Akun nonaktif" / "Hubungi admin Ruang Sehat.", KU-10).
 * Pemicu pengalihan ke sini belum ada `[TIDAK DIKETAHUI]`.
 */
export default function AkunNonaktif() {
  return (
    <LayarStatus judul="Akun nonaktif">
      <Alert variant="warning" title="Akun nonaktif">
        Hubungi admin Ruang Sehat.
      </Alert>
    </LayarStatus>
  );
}
