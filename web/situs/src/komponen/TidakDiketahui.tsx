/** Penanda teks yang belum ditetapkan (anti-halu): tampil jelas, bukan konten karangan. */
export function TidakDiketahui() {
  return <span className="situs-td">[TIDAK DIKETAHUI]</span>;
}

/** Teks CMS atau, bila `null`, penanda `[TIDAK DIKETAHUI]` dengan keterangan bagian yang kosong. */
export function TeksAtauTidakDiketahui({ teks, ganti }: { teks: string | null; ganti: string }) {
  if (teks !== null) return <>{teks}</>;
  return (
    <>
      {ganti} <TidakDiketahui />
    </>
  );
}
