"use client";

import { Alert, Badge, Button, Card, Skeleton } from "@rs/ui";
import Link from "next/link";
import { useCallback, useEffect, useMemo, useState } from "react";
import { isiBeranda } from "../../lib/beranda/model.ts";
import type { Gagal } from "../../lib/hr/klien.ts";
import { buatKlienVerifikasi, type KlienVerifikasi, type Muatan } from "../../lib/verifikasi/klien.ts";
import { GalatAkses, adalahGalatAkses } from "../hr/GalatAkses.tsx";

export interface BerandaProps {
  readonly csrfToken: string;
  /** Hanya untuk test. */
  readonly klien?: KlienVerifikasi;
  readonly arahkan?: (url: string) => void;
}

/**
 * Beranda HR (CR-039, Q32): satu hal, yaitu status perusahaan dan satu tombol langkah berikutnya (bukan dasbor
 * angka). Sumber data satu-satunya `GET /v1/corporate/documents` (status, kelengkapan dokumen; 409 `4090112` =
 * Aktif). Rincian status, dokumen, dan koreksi data tetap di `/verifikasi`, dicapai dari tombol ini. Tidak pernah
 * menampilkan data kesehatan peserta (BR-143). Otorisasi tetap di API (ADR-012).
 */
export function Beranda({ csrfToken, klien, arahkan }: BerandaProps) {
  const api = useMemo(() => klien ?? buatKlienVerifikasi(csrfToken), [klien, csrfToken]);
  const [muatan, setMuatan] = useState<Muatan | null>(null);
  const [galat, setGalat] = useState<Gagal | null>(null);
  const [memuat, setMemuat] = useState(false);

  const muat = useCallback(async () => {
    setMemuat(true);
    const h = await api.muat();
    setMemuat(false);
    if (h.ok) {
      setMuatan(h.data);
      setGalat(null);
    } else {
      setGalat(h);
    }
  }, [api]);

  useEffect(() => {
    void muat();
  }, [muat]);

  if (adalahGalatAkses(galat)) return <GalatAkses galat={galat} {...(arahkan === undefined ? {} : { arahkan })} />;

  const isi = muatan === null ? null : isiBeranda(muatan);
  return (
    <section aria-labelledby="judul-beranda" className="kp-bagian">
      <h1 id="judul-beranda">Beranda</h1>
      {galat === null ? null : (
        <Alert variant="danger" title="Beranda belum bisa dimuat">
          <p>{galat.pesan}</p>
          <Button variant="secondary" loading={memuat} loadingLabel="Memuat..." onClick={() => void muat()}>
            Coba lagi
          </Button>
        </Alert>
      )}
      {muatan === null && galat === null ? <Skeleton lines={3} /> : null}
      {isi === null ? null : (
        <Card>
          <div className="kp-beranda" data-status={isi.kunci}>
            <p className="kp-baris-status">
              <span className="kp-kecil">Status perusahaan</span>
              <Badge tone={isi.status.tone}>{isi.status.label}</Badge>
            </p>
            <h2>{isi.judul}</h2>
            {isi.kalimat.map((k) => (
              <p key={k}>{k}</p>
            ))}
            <p>
              <Link href={isi.tindakan.href} prefetch={false} className="rs-btn rs-btn--primary kp-aksi-utama">
                {isi.tindakan.label}
              </Link>
            </p>
          </div>
        </Card>
      )}
    </section>
  );
}
