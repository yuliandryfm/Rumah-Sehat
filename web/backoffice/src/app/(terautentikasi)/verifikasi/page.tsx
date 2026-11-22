import { PanelVerifikasi } from "../../../komponen/verifikasi/PanelVerifikasi.tsx";
import { wajibSesi } from "../../../lib/server/sesi.ts";

export default async function HalamanVerifikasi() {
  const { csrfToken } = await wajibSesi();
  return <PanelVerifikasi csrfToken={csrfToken} />;
}
