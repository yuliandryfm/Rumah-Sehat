"use client";

import { Alert, Badge, Button, Card } from "@rs/ui";
import Link from "next/link";
import { useCallback, useEffect, useId, useMemo, useState } from "react";
import { buatKlienCms, type Gagal, type KlienCms } from "../../lib/cms/klien.ts";
import {
  LABEL_HALAMAN,
  LABEL_JENIS,
  LABEL_KANAL,
  LABEL_STATUS,
  NADA_STATUS,
  URUTAN_HALAMAN,
  URUTAN_KANAL,
  URUTAN_STATUS,
  bolehDihapus,
  bolehDiubah,
  formatWaktu,
  type HalamanSitus,
  type Kanal,
  type Konten,
  type StatusTampil,
} from "../../lib/cms/model.ts";
import { DialogAlasan } from "./DialogAlasan.tsx";
import { LayarTanpaAkses } from "./LayarTanpaAkses.tsx";

export interface DaftarKontenProps {
  readonly csrfToken: string;
  /** Hanya untuk test. */
  readonly klien?: KlienCms;
}

function Pilihan<T extends string>({
  label,
  nilai,
  opsi,
  semua,
  onUbah,
}: {
  label: string;
  nilai: T | "";
  opsi: readonly { v: T; l: string }[];
  semua: string;
  onUbah: (v: T | "") => void;
}) {
  const id = useId();
  return (
    <div className="rs-field bo-filter">
      <label className="rs-field__label" htmlFor={id}>
        {label}
      </label>
      <select id={id} className="rs-field__input" value={nilai} onChange={(e) => { onUbah(e.target.value as T | ""); }}>
        <option value="">{semua}</option>
        {opsi.map((o) => (
          <option key={o.v} value={o.v}>
            {o.l}
          </option>
        ))}
      </select>
    </div>
  );
}

/** Layar daftar konten (D-C07 butir 1, wireframe C14 layar 1): filter, hapus dengan alasan, pulihkan. */
export function DaftarKonten({ csrfToken, klien }: DaftarKontenProps) {
  const api = useMemo(() => klien ?? buatKlienCms(csrfToken), [klien, csrfToken]);
  const [kanal, setKanal] = useState<Kanal | "">("");
  const [status, setStatus] = useState<StatusTampil | "">("");
  const [halaman, setHalaman] = useState<HalamanSitus | "">("");
  const [item, setItem] = useState<readonly Konten[] | null>(null);
  const [berikut, setBerikut] = useState<string | null>(null);
  const [galat, setGalat] = useState<Gagal | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [hapusId, setHapusId] = useState<string | null>(null);
  const [sibuk, setSibuk] = useState(false);

  const muat = useCallback(
    async (cursor?: string) => {
      const h = await api.daftar({
        ...(status === "" ? {} : { status }),
        ...(kanal === "" ? {} : { channel: kanal }),
        ...(halaman === "" ? {} : { site_page: halaman }),
        ...(cursor === undefined ? {} : { cursor }),
      });
      if (!h.ok) {
        setGalat(h);
        setItem(null);
        return;
      }
      setGalat(null);
      setItem((lama) => (cursor === undefined || lama === null ? h.data.items : [...lama, ...h.data.items]));
      setBerikut(h.data.hasMore ? h.data.nextCursor : null);
    },
    [api, status, kanal, halaman],
  );

  useEffect(() => {
    setItem(null);
    void muat();
  }, [muat]);

  if (galat?.status === 403) return <LayarTanpaAkses />;

  const hapus = async (alasan: string) => {
    if (hapusId === null) return;
    setSibuk(true);
    const h = await api.hapus(hapusId, alasan);
    setSibuk(false);
    if (!h.ok) {
      setGalat(h);
      return;
    }
    setHapusId(null);
    setInfo("Konten dihapus. Konten dapat dipulihkan dari filter Terhapus.");
    await muat();
  };

  const pulihkan = async (id: string) => {
    setSibuk(true);
    const h = await api.pulihkan(id);
    setSibuk(false);
    if (!h.ok) {
      setGalat(h);
      return;
    }
    setInfo("Konten dipulihkan.");
    await muat();
  };

  const terhapus = status === "deleted";
  const target = item?.find((k) => k.content_id === hapusId);

  return (
    <>
      <div className="bo-judul-baris">
        <h1>Daftar konten</h1>
        <Link className="rs-btn rs-btn--primary" href="/konten/baru">
          Buat konten
        </Link>
      </div>
      <p className="bo-bantu">Promo hanya informasi dan tidak mengubah harga di aplikasi maupun katalog korporat.</p>
      <div className="bo-filter-baris" role="group" aria-label="Saring konten">
        <Pilihan label="Kanal" nilai={kanal} semua="Semua" opsi={URUTAN_KANAL.map((v) => ({ v, l: LABEL_KANAL[v] }))} onUbah={setKanal} />
        <Pilihan label="Status" nilai={status} semua="Semua" opsi={URUTAN_STATUS.map((v) => ({ v, l: LABEL_STATUS[v] }))} onUbah={setStatus} />
        <Pilihan label="Halaman situs" nilai={halaman} semua="Semua" opsi={URUTAN_HALAMAN.map((v) => ({ v, l: LABEL_HALAMAN[v] }))} onUbah={setHalaman} />
      </div>
      {info === null ? null : (
        <Alert variant="success" title={info} />
      )}
      {galat === null ? null : (
        <Alert variant="danger" title="Tindakan gagal">
          {galat.pesan}
        </Alert>
      )}
      {hapusId === null ? null : (
        <DialogAlasan
          judul={`Hapus konten "${target?.title ?? ""}"?`}
          penjelasan="Konten tidak tampil lagi di kanal terpilih dan dapat dipulihkan dari filter Terhapus. Tindakan ini dicatat."
          labelAksi="Hapus"
          sibuk={sibuk}
          onBatal={() => { setHapusId(null); }}
          onKirim={(a) => void hapus(a)}
        />
      )}
      {item === null && galat === null ? (
        <Card loading aria-label="Memuat daftar konten" />
      ) : item !== null && item.length === 0 ? (
        <Card>
          <h2>Tidak ada konten</h2>
          <p>Belum ada konten untuk filter ini.</p>
        </Card>
      ) : item === null ? null : (
        <div className="bo-tabel-bungkus">
          <table className="bo-tabel">
            <caption className="bo-sr">Daftar konten</caption>
            <thead>
              {terhapus ? (
                <tr>
                  <th scope="col">Judul</th>
                  <th scope="col">Dihapus</th>
                  <th scope="col">Alasan</th>
                  <th scope="col">Aksi</th>
                </tr>
              ) : (
                <tr>
                  <th scope="col">Judul</th>
                  <th scope="col">Jenis</th>
                  <th scope="col">Kanal</th>
                  <th scope="col">Halaman situs</th>
                  <th scope="col">Status</th>
                  <th scope="col">Tayang sejak</th>
                  <th scope="col">Aksi</th>
                </tr>
              )}
            </thead>
            <tbody>
              {item.map((k) =>
                terhapus ? (
                  <tr key={k.content_id}>
                    <th scope="row">{k.title}</th>
                    <td>{formatWaktu(k.deleted_at)}</td>
                    <td>{k.delete_reason ?? "-"}</td>
                    <td>
                      <Button variant="secondary" disabled={sibuk} onClick={() => void pulihkan(k.content_id)}>
                        Pulihkan
                      </Button>
                    </td>
                  </tr>
                ) : (
                  <tr key={k.content_id}>
                    <th scope="row">{k.title}</th>
                    <td>{LABEL_JENIS[k.content_type]}</td>
                    <td>{LABEL_KANAL[k.channel]}</td>
                    <td>{k.site_page === null || k.site_page === undefined ? "-" : LABEL_HALAMAN[k.site_page]}</td>
                    <td>
                      <Badge tone={NADA_STATUS[k.display_status]}>{LABEL_STATUS[k.display_status]}</Badge>
                    </td>
                    <td>{formatWaktu(k.publish_start_at)}</td>
                    <td className="bo-aksi">
                      <Link href={`/konten/${k.content_id}`} className="rs-btn rs-btn--secondary">
                        {bolehDiubah(k) ? "Ubah" : "Lihat"}
                      </Link>
                      {bolehDihapus(k) ? (
                        <Button variant="text" onClick={() => { setHapusId(k.content_id); }}>
                          Hapus
                        </Button>
                      ) : null}
                    </td>
                  </tr>
                ),
              )}
            </tbody>
          </table>
          {berikut === null ? null : (
            <Button variant="secondary" onClick={() => void muat(berikut)}>
              Muat lebih banyak
            </Button>
          )}
        </div>
      )}
    </>
  );
}
