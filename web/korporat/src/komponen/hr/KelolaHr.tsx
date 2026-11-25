"use client";

import { Alert, Badge, Button, Skeleton } from "@rs/ui";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  KODE_SUDAH_NONAKTIF,
  buatKlienHr,
  type AnggotaHr,
  type Gagal,
  type KlienHr,
} from "../../lib/hr/klien.ts";
import { LABEL_STATUS, dapatDinonaktifkan, namaTampil } from "../../lib/hr/model.ts";
import { GalatAkses, adalahGalatAkses } from "./GalatAkses.tsx";

export interface KelolaHrProps {
  readonly csrfToken: string;
  /** Hanya untuk test. */
  readonly klien?: KlienHr;
  readonly arahkan?: (url: string) => void;
}

/**
 * Kelola HR (wireframe K01 layar 10, US-1904 AC2/AC3/AC5): daftar anggota HR perusahaan pemanggil dan
 * penonaktifan dengan konfirmasi dampak. Penolakan invarian server (HR aktif terakhir 4090105, diri sendiri
 * 4090101) ditampilkan apa adanya sebagai galat, bukan dicegah diam-diam di klien: server yang memutuskan.
 * Konfirmasi memakai panel sebaris (bukan dialog modal): fokus pindah ke panel dan kembali ke tombol pemicu
 * saat dibatalkan. "Undang HR" belum tersedia: backend undangan menunggu ADR-030/CR-026, tidak dipalsukan.
 */
export function KelolaHr({ csrfToken, klien, arahkan }: KelolaHrProps) {
  const api = useMemo(() => klien ?? buatKlienHr(csrfToken), [klien, csrfToken]);
  const [anggota, setAnggota] = useState<readonly AnggotaHr[] | null>(null);
  const [galat, setGalat] = useState<Gagal | null>(null);
  const [galatAksi, setGalatAksi] = useState<Gagal | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [target, setTarget] = useState<AnggotaHr | null>(null);
  const [sibuk, setSibuk] = useState(false);
  const panel = useRef<HTMLDivElement>(null);
  const pemicu = useRef<HTMLElement | null>(null);

  const muat = useCallback(async () => {
    const h = await api.daftar();
    if (h.ok) {
      setAnggota(h.data);
      setGalat(null);
    } else {
      setGalat(h);
    }
  }, [api]);

  useEffect(() => {
    void muat();
  }, [muat]);

  useEffect(() => {
    if (target !== null) panel.current?.focus();
  }, [target]);

  if (adalahGalatAkses(galat)) return <GalatAkses galat={galat} {...(arahkan === undefined ? {} : { arahkan })} />;
  // 401 saat aksi (sesi dicabut atau akun nonaktif di tengah jalan) sama dengan saat memuat.
  if (adalahGalatAkses(galatAksi)) return <GalatAkses galat={galatAksi} {...(arahkan === undefined ? {} : { arahkan })} />;

  const mulaiNonaktif = (a: AnggotaHr, el: HTMLElement) => {
    pemicu.current = el;
    setInfo(null);
    setGalatAksi(null);
    setTarget(a);
  };
  const batal = () => {
    setTarget(null);
    pemicu.current?.focus();
  };
  const nonaktifkan = async () => {
    if (target === null) return;
    setSibuk(true);
    const h = await api.nonaktifkan(target.corporate_member_id);
    setSibuk(false);
    if (!h.ok) {
      setGalatAksi(h);
      // Daftar basi (mis. HR sudah dinonaktifkan pihak lain): muat ulang supaya tampilan sesuai server.
      if (h.kode === KODE_SUDAH_NONAKTIF || h.status === 404) await muat();
      setTarget(null);
      return;
    }
    setTarget(null);
    setInfo(
      h.data.identity_provider_synced
        ? `${namaTampil(target)} dinonaktifkan dan tidak dapat masuk lagi.`
        : `${namaTampil(target)} dinonaktifkan. Akses sudah ditutup; sesi masuknya diakhiri menyusul.`,
    );
    await muat();
  };

  return (
    <>
      <div className="kp-baris">
        <h1>Kelola HR</h1>
        <div className="kp-aksi-judul">
          <Button variant="primary" disabled aria-describedby="alasan-undang">
            Undang HR
          </Button>
          <span id="alasan-undang" className="kp-kecil">
            Belum tersedia: undangan HR baru menunggu keputusan arsitektur (CR-026).
          </span>
        </div>
      </div>

      {info === null ? null : <Alert variant="success" title="Berhasil">{info}</Alert>}
      {galat === null ? null : <Alert variant="danger" title="Daftar HR tidak dapat dimuat">{galat.pesan}</Alert>}
      {galatAksi === null ? null : (
        <Alert variant="danger" title="Tidak dapat dinonaktifkan">
          {galatAksi.pesan}
        </Alert>
      )}

      {anggota === null && galat === null ? <Skeleton lines={4} /> : null}
      {anggota === null ? null : (
        <div className="kp-tabel-bungkus">
          <table className="kp-tabel">
            <caption>Anggota HR perusahaan. Semua HR berperan sama.</caption>
            <thead>
              <tr>
                <th scope="col">Nama</th>
                <th scope="col">Email</th>
                <th scope="col">Status</th>
                <th scope="col">Aksi</th>
              </tr>
            </thead>
            <tbody>
              {anggota.map((a) => {
                const st = LABEL_STATUS[a.status];
                return (
                  <tr key={a.corporate_member_id}>
                    <td>
                      {namaTampil(a)}
                      {a.is_self ? " (Anda)" : ""}
                    </td>
                    <td>{a.email}</td>
                    <td>
                      <Badge tone={st.tone}>{st.label}</Badge>
                    </td>
                    <td>
                      {dapatDinonaktifkan(a) ? (
                        <Button
                          variant="secondary"
                          aria-label={`Nonaktifkan ${namaTampil(a)}`}
                          onClick={(e) => {
                            mulaiNonaktif(a, e.currentTarget);
                          }}
                        >
                          Nonaktifkan
                        </Button>
                      ) : (
                        <span className="kp-kecil">
                          {a.is_self ? "Tidak dapat menonaktifkan akun sendiri" : "-"}
                        </span>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {target === null ? null : (
        <div ref={panel} tabIndex={-1} role="group" aria-labelledby="judul-konfirmasi" className="kp-konfirmasi">
          <h2 id="judul-konfirmasi">Nonaktifkan {namaTampil(target)}?</h2>
          <Alert variant="warning" title="Dampak penonaktifan">
            {namaTampil(target)} tidak dapat masuk lagi dan sesi yang terbuka diakhiri pada permintaan berikutnya.
          </Alert>
          <div className="kp-baris-aksi">
            <Button variant="secondary" onClick={batal} disabled={sibuk}>
              Batal
            </Button>
            <Button variant="destructive" loading={sibuk} loadingLabel="Menonaktifkan..." onClick={() => void nonaktifkan()}>
              Nonaktifkan
            </Button>
          </div>
        </div>
      )}
    </>
  );
}
