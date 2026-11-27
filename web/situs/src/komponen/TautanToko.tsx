import Link from "next/link";
import { ambilUrlToko, type NamaToko } from "../konten/tautan-toko.ts";

const TOKO: readonly { id: NamaToko; nama: string }[] = [
  { id: "playstore", nama: "Google Play" },
  { id: "appstore", nama: "App Store" },
];

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
            {t.nama}
          </Link>
        ) : (
          <a key={t.id} className="situs-toko__tautan" href={url} rel="noopener noreferrer" aria-label={`Unduh di ${t.nama}`}>
            {t.nama}
          </a>
        );
      })}
    </div>
  );
}
