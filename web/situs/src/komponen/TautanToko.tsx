import Link from "next/link";
import { ambilUrlToko, type NamaToko } from "../konten/tautan-toko.ts";

const TOKO: readonly { id: NamaToko; nama: string; awal: string }[] = [
  { id: "playstore", nama: "Google Play", awal: "Google " },
  { id: "appstore", nama: "App Store", awal: "" },
];

/** Teks tampak; di ponsel awalan "Google " disembunyikan CSS (nama aksesibel tetap lengkap lewat aria-label). */
function label(t: { nama: string; awal: string }) {
  if (t.awal === "") return t.nama;
  return (
    <>
      <span className="situs-toko__awal">{t.awal}</span>
      {t.nama.slice(t.awal.length)}
    </>
  );
}

/**
 * Tautan toko aplikasi di pojok kanan header (CR-038). Alamat toko dari env (`tautan-toko.ts`); bila belum
 * diatur atau tidak valid, mengarah ke halaman Unduh aplikasi. Sementara ini berupa teks; lencana resmi
 * Google Play dan App Store menunggu aset resmi dan persetujuan lisensinya (KEPUTUSAN-422).
 */
export function TautanToko() {
  return (
    <div className="situs-toko" role="group" aria-label="Unduh aplikasi">
      {TOKO.map((t) => {
        const url = ambilUrlToko(t.id);
        return url === null ? (
          <Link key={t.id} className="situs-toko__tautan" href="/unduh" prefetch={false} aria-label={`${t.nama}: lihat halaman Unduh aplikasi`}>
            {label(t)}
          </Link>
        ) : (
          <a key={t.id} className="situs-toko__tautan" href={url} rel="noopener noreferrer" aria-label={`Unduh di ${t.nama}`}>
            {label(t)}
          </a>
        );
      })}
    </div>
  );
}
