"use client";

import { Alert, Badge, Card, Skeleton } from "@rs/ui";
import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import { buatKlienHr, type AnggotaHr, type Gagal, type KlienHr } from "../../lib/hr/klien.ts";
import { ringkas } from "../../lib/hr/model.ts";
import { GalatAkses, adalahGalatAkses } from "./GalatAkses.tsx";

export interface DasborHrProps {
  readonly csrfToken: string;
  /** Hanya untuk test. */
  readonly klien?: KlienHr;
  readonly arahkan?: (url: string) => void;
}

/**
 * Dasbor HR Fase 1 (wireframe K01 layar 9): status perusahaan, jumlah HR aktif, tautan Kelola HR.
 * Satu-satunya sumber data adalah daftar anggota HR: API menjawab 200 hanya bila perusahaan Aktif, jadi
 * "Aktif" disimpulkan dari 200 dan perusahaan lain statusnya tampil lewat layar akses (401/403). Nama
 * perusahaan dan identitas HR belum tampil: endpoint profil belum ada `[TIDAK DIKETAHUI]`. Tidak pernah
 * menampilkan data kesehatan peserta (BR-143).
 */
export function DasborHr({ csrfToken, klien, arahkan }: DasborHrProps) {
  const api = useMemo(() => klien ?? buatKlienHr(csrfToken), [klien, csrfToken]);
  const [anggota, setAnggota] = useState<readonly AnggotaHr[] | null>(null);
  const [galat, setGalat] = useState<Gagal | null>(null);

  useEffect(() => {
    let batal = false;
    void api.daftar().then((h) => {
      if (batal) return;
      if (h.ok) {
        setAnggota(h.data);
        setGalat(null);
      } else {
        setGalat(h);
      }
    });
    return () => {
      batal = true;
    };
  }, [api]);

  if (adalahGalatAkses(galat)) return <GalatAkses galat={galat} {...(arahkan === undefined ? {} : { arahkan })} />;

  return (
    <>
      <h1>Dasbor</h1>
      {galat === null ? null : <Alert variant="danger" title="Dasbor tidak dapat dimuat">{galat.pesan}</Alert>}
      {anggota === null && galat === null ? <Skeleton lines={3} /> : null}
      {anggota === null ? null : (
        <>
          <Alert variant="success" title="Perusahaan terverifikasi">
            Status: Aktif. Akun HR Anda dapat memakai menu pesanan saat tersedia.
          </Alert>
          <div className="kp-kartu">
            <Card>
              <span className="kp-kecil">Status perusahaan</span>
              <Badge tone="success">Aktif</Badge>
            </Card>
            <Card>
              <span className="kp-kecil">HR aktif</span>
              <span className="kp-angka" data-testid="jumlah-hr-aktif">{ringkas(anggota).aktif}</span>
              <Link href="/kelola-hr" prefetch={false}>Kelola HR</Link>
            </Card>
            <Card>
              <span className="kp-kecil">Pesanan berjalan</span>
              <span className="kp-kecil">Belum tersedia (layanan korporat dibuka di fase berikutnya).</span>
            </Card>
          </div>
        </>
      )}
    </>
  );
}
