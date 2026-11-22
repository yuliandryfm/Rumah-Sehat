import { DaftarKonten } from "../../../komponen/cms/DaftarKonten.tsx";
import { wajibSesi } from "../../../lib/server/sesi.ts";

export default async function HalamanDaftarKonten() {
  const { csrfToken } = await wajibSesi();
  return <DaftarKonten csrfToken={csrfToken} />;
}
