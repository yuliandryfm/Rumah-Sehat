import { headers } from "next/headers";
import { idKorelasi } from "../korelasi.ts";

/** ID korelasi permintaan ini (dipasang `proxy.ts`), untuk kode referensi di layar server (KEPUTUSAN-172). */
export async function korelasiPermintaan(): Promise<string> {
  return idKorelasi((await headers()).get("x-correlation-id"));
}
