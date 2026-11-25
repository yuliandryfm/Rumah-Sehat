"use client";

import { Alert } from "@rs/ui";
import { useEffect } from "react";
import { KODE_AKUN_NONAKTIF, KODE_CSRF_DITOLAK, type Gagal } from "../../lib/hr/klien.ts";

/** Kontak Layanan Pelanggan belum ditetapkan `[TIDAK DIKETAHUI]` (brief web korporat §3); tidak dikarang. */
const KONTAK_BELUM_ADA = "Hubungi Layanan Pelanggan Ruang Sehat.";

/**
 * Layar akun nonaktif untuk pemegang sesi yang sah (KU-10, US-1904 AC5): hanya muncul dari 401 `4010002`.
 * Jalur masuk memakai pesan seragam (`/akun-nonaktif` publik, UP-73), jadi status akun tidak bocor ke orang
 * yang belum berhasil masuk. Dirender di dalam `<main>` kerangka, bukan `<main>` sendiri.
 */
export function LayarAkunNonaktif() {
  return (
    <section aria-labelledby="judul-akun-nonaktif" className="kp-bagian">
      <h1 id="judul-akun-nonaktif">Akun nonaktif</h1>
      <Alert variant="danger" title="Akun atau perusahaan Anda dinonaktifkan">
        Anda tidak dapat memakai member area lagi dengan akun ini. {KONTAK_BELUM_ADA}
      </Alert>
    </section>
  );
}

/**
 * Akses ditolak: perusahaan belum Aktif (menunggu verifikasi atau ditolak) → API menjawab 403 (US-1901 AC6).
 * Status dan alasan penolakan belum dapat ditampilkan: API Rust belum punya endpoint profil/status
 * perusahaan untuk HR `[TIDAK DIKETAHUI]`, jadi layar ini tidak menebaknya.
 */
export function LayarPerusahaanBelumAktif() {
  return (
    <section aria-labelledby="judul-belum-aktif" className="kp-bagian">
      <h1 id="judul-belum-aktif">Menu ini belum dapat dipakai</h1>
      <Alert variant="warning" title="Perusahaan Anda belum aktif">
        Menu ini terbuka setelah perusahaan Anda diverifikasi dan berstatus Aktif. Bila Anda perlu bantuan, {KONTAK_BELUM_ADA.toLowerCase()}
      </Alert>
    </section>
  );
}

function Sesi401({ arahkan }: { arahkan: (url: string) => void }) {
  // BFF sudah menghapus sesi dan cookie saat API menjawab 401: tidak ada yang tersisa selain masuk lagi.
  useEffect(() => {
    arahkan("/auth/login");
  }, [arahkan]);
  return (
    <section aria-labelledby="judul-sesi" className="kp-bagian">
      <h1 id="judul-sesi">Sesi berakhir</h1>
      <Alert variant="info" title="Masuk lagi untuk melanjutkan">
        Anda dialihkan ke halaman masuk. <a href="/auth/login">Masuk lagi</a>
      </Alert>
    </section>
  );
}

const arahkanBawaan = (url: string): void => {
  window.location.assign(url);
};

/**
 * Memetakan galat akses ke layar: 401 `4010002` → akun nonaktif, 401 lain → alihkan ke login, 403 → perusahaan
 * belum aktif, kecuali 403 `4030101` (penolakan CSRF/Origin oleh BFF) yang ditampilkan pemanggil sebagai pesan biasa. Galat lain: `null` (pemanggil menampilkan pesan sendiri). Bukan kontrol akses (ADR-012).
 */
export function GalatAkses({ galat, arahkan = arahkanBawaan }: { galat: Gagal | null; arahkan?: (url: string) => void }) {
  if (galat === null) return null;
  if (galat.status === 401) return galat.kode === KODE_AKUN_NONAKTIF ? <LayarAkunNonaktif /> : <Sesi401 arahkan={arahkan} />;
  if (galat.status === 403 && galat.kode !== KODE_CSRF_DITOLAK) return <LayarPerusahaanBelumAktif />;
  return null;
}

export const adalahGalatAkses = (g: Gagal | null): boolean =>
  g !== null && (g.status === 401 || (g.status === 403 && g.kode !== KODE_CSRF_DITOLAK));
