import { PanelPerusahaan } from "../../../komponen/perusahaan/PanelPerusahaan.tsx";
import { wajibSesi } from "../../../lib/server/sesi.ts";

export default async function HalamanVerifikasiPerusahaan() {
  const { csrfToken } = await wajibSesi();
  return <PanelPerusahaan csrfToken={csrfToken} />;
}
