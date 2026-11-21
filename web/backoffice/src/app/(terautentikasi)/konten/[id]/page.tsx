import { EditorKonten } from "../../../../komponen/cms/EditorKonten.tsx";
import { wajibSesi } from "../../../../lib/server/sesi.ts";

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
/** UUID nil: pasti tidak ada, jadi API menjawab 404 dan editor menampilkan "Konten tidak ditemukan". */
const UUID_NIL = "00000000-0000-0000-0000-000000000000";

export default async function HalamanUbahKonten({ params }: { params: Promise<{ id: string }> }) {
  const { csrfToken } = await wajibSesi();
  const { id } = await params;
  // ID yang bukan UUID tidak pernah diteruskan ke API.
  return <EditorKonten csrfToken={csrfToken} kontenId={UUID.test(id) ? id : UUID_NIL} />;
}
