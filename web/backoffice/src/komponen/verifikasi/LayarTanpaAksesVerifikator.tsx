import { Alert } from "@rs/ui";

/** API menjawab 403 untuk akun tanpa sub-peran Verifikator (US-201 AC6): tidak ada data yang ditampilkan. */
export function LayarTanpaAksesVerifikator() {
  return (
    <>
      <h1>Antrean verifikasi identitas</h1>
      <Alert variant="warning" title="Tidak ada akses">
        Akun Anda tidak memiliki sub-peran Verifikator untuk memeriksa dokumen identitas.
      </Alert>
    </>
  );
}
