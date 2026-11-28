import { TautanToko } from "./TautanToko.tsx";
import { VarianBand } from "./VarianBand.tsx";

/**
 * Pita ajakan unduh di akhir isi setiap halaman (CR-042, desain Stitch "App Download CTA Band"). Tautan toko memakai
 * logika CR-038 (alamat dari env, jatuh ke halaman Unduh aplikasi bila belum diatur) dan dirender di server;
 * teks per halaman dipilih oleh `VarianBand` (KEPUTUSAN-491).
 */
export function BandUnduh() {
  return (
    <VarianBand>
      <TautanToko varian="band" />
    </VarianBand>
  );
}
