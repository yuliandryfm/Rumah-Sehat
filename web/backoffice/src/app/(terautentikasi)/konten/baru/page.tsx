import { EditorKonten } from "../../../../komponen/cms/EditorKonten.tsx";
import { wajibSesi } from "../../../../lib/server/sesi.ts";

export default async function HalamanKontenBaru() {
  const { csrfToken } = await wajibSesi();
  return <EditorKonten csrfToken={csrfToken} />;
}
