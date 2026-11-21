import { Alert } from "@rs/ui";

/** Layar "Tidak ada akses" (wireframe C14): API menjawab 403 untuk akun tanpa sub-peran Konten. */
export function LayarTanpaAkses() {
  return (
    <>
      <h1>Konten</h1>
      <Alert variant="warning" title="Tidak ada akses">
        Akun Anda tidak memiliki sub-peran Konten untuk membuat, mengubah, atau menayangkan konten.
      </Alert>
    </>
  );
}
