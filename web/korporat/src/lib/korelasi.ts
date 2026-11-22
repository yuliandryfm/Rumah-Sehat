const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/** ID korelasi: nilai masuk dipakai hanya bila UUID sah (sama dengan `@rs/bff-auth`); bila tidak, UUID baru. */
export function idKorelasi(masuk: string | null): string {
  return masuk !== null && UUID.test(masuk) ? masuk.toLowerCase() : crypto.randomUUID();
}

/**
 * Kode referensi untuk layar galat (KEPUTUSAN-172, ADR-023): `correlation_id` bila ada pada galat,
 * selain itu `digest` galat Next.js (hanya galat render server; bukan `correlation_id` respons API).
 * Nilai disaring (huruf, angka, strip; 6-64 karakter) supaya tidak pernah menampilkan teks galat.
 */
export function kodeReferensi(galat: { digest?: string | undefined; correlationId?: unknown }): string | null {
  for (const k of [galat.correlationId, galat.digest]) {
    if (typeof k === "string" && /^[A-Za-z0-9-]{6,64}$/.test(k)) return k;
  }
  return null;
}
