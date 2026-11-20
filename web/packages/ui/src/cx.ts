/** Gabungkan nama kelas CSS; nilai kosong dan `false` dibuang. */
export function cx(...bagian: (string | false | null | undefined)[]): string {
  return bagian.filter((b): b is string => typeof b === "string" && b.length > 0).join(" ");
}
