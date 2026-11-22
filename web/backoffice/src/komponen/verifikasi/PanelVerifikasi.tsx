"use client";

import { Alert, Badge, Button, Card } from "@rs/ui";
import { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";
import { buatKlienVerifikasi, type Gagal, type KlienVerifikasi } from "../../lib/verifikasi/klien.ts";
import {
  BATAS_KETERANGAN,
  KODE_LAINNYA,
  KODE_NIK_TIDAK_TERSEDIA,
  LABEL_DOKUMEN,
  LABEL_SEX,
  LABEL_STATUS,
  formatTanggal,
  formatWib,
  nikTermasking,
  type AlasanTolak,
  type Detail,
  type FotoDokumen,
  type ItemAntrean,
} from "../../lib/verifikasi/model.ts";
import { LayarTanpaAksesVerifikator } from "./LayarTanpaAksesVerifikator.tsx";

export interface PanelVerifikasiProps {
  readonly csrfToken: string;
  /** Hanya untuk test. */
  readonly klien?: KlienVerifikasi;
}

const KODE_PEMISAHAN_TUGAS = "4090205";
const KODE_NIK_TERPAKAI = "4090203";
const KODE_SUDAH_DIPUTUSKAN = "4090204";

function dokumen(t: string): string {
  return LABEL_DOKUMEN[t] ?? t.toUpperCase();
}

/**
 * Foto dokumen (ADR-018 butir 3, US-201 AC6). Hanya dari URL unduh bertanda tangan berumur pendek yang
 * diminta eksplisit (E-05 tercatat sebelum URL terbit). URL hanya di memori komponen: tidak ditulis ke
 * storage, tidak dicatat, `referrerPolicy="no-referrer"`, dan dilepas saat kedaluwarsa atau panel ditutup.
 */
function FotoDokumenView({ foto, jenis, onKedaluwarsa }: { foto: FotoDokumen; jenis: string; onKedaluwarsa: () => void }) {
  const [gagalMuat, setGagalMuat] = useState(false);
  const kedaluwarsa = useRef(onKedaluwarsa);
  kedaluwarsa.current = onKedaluwarsa;
  useEffect(() => {
    const sisa = new Date(foto.expires_at).getTime() - Date.now();
    if (Number.isNaN(sisa) || sisa <= 0) {
      kedaluwarsa.current();
      return;
    }
    const t = setTimeout(() => { kedaluwarsa.current(); }, sisa);
    return () => { clearTimeout(t); };
  }, [foto.expires_at]);
  if (gagalMuat) {
    return (
      <Alert variant="warning" title="Foto tidak dapat dimuat">
        Tautan foto berumur pendek dan mungkin sudah kedaluwarsa. Tampilkan foto lagi.
      </Alert>
    );
  }
  return (
    <img
      className="bo-foto"
      src={foto.url}
      alt={`Foto ${dokumen(jenis)} pengajuan`}
      referrerPolicy="no-referrer"
      onError={() => { setGagalMuat(true); }}
    />
  );
}

/** Layar antrean verifikasi identitas (wireframe C02, US-201): daftar terlama dulu dan panel detail. */
export function PanelVerifikasi({ csrfToken, klien }: PanelVerifikasiProps) {
  const api = useMemo(() => klien ?? buatKlienVerifikasi(csrfToken), [klien, csrfToken]);
  const [item, setItem] = useState<readonly ItemAntrean[] | null>(null);
  const [berikut, setBerikut] = useState<string | null>(null);
  const [alasan, setAlasan] = useState<readonly AlasanTolak[]>([]);
  const [galat, setGalat] = useState<Gagal | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [bukaId, setBukaId] = useState<string | null>(null);
  const [detail, setDetail] = useState<Detail | null>(null);
  const [galatPanel, setGalatPanel] = useState<Gagal | null>(null);
  const [foto, setFoto] = useState<FotoDokumen | null>(null);
  const [sibuk, setSibuk] = useState(false);
  const [kodeAlasan, setKodeAlasan] = useState("");
  const [keterangan, setKeterangan] = useState("");
  const [galatIsian, setGalatIsian] = useState<string | null>(null);
  const [terkunci, setTerkunci] = useState(false);
  const uid = useId();

  const tutupPanel = useCallback(() => {
    // Data sensitif (NIK lengkap, URL foto) dibuang dari memori saat panel ditutup.
    setBukaId(null);
    setDetail(null);
    setFoto(null);
    setGalatPanel(null);
    setKodeAlasan("");
    setKeterangan("");
    setGalatIsian(null);
    setTerkunci(false);
  }, []);

  const muat = useCallback(
    async (cursor?: string) => {
      const h = await api.antrean(cursor);
      if (!h.ok) {
        setGalat(h);
        if (cursor === undefined) setItem(null);
        return;
      }
      setGalat(null);
      setItem((lama) => (cursor === undefined || lama === null ? h.data.items : [...lama, ...h.data.items]));
      setBerikut(h.data.hasMore ? h.data.nextCursor : null);
    },
    [api],
  );

  useEffect(() => {
    void muat();
    void api.alasan().then((h) => {
      if (h.ok) setAlasan(h.data);
      else setGalat((g) => g ?? h);
    });
  }, [api, muat]);

  if (galat?.status === 403) return <LayarTanpaAksesVerifikator />;

  const buka = async (id: string) => {
    tutupPanel();
    setInfo(null);
    setBukaId(id);
    setSibuk(true);
    const h = await api.detail(id);
    setSibuk(false);
    if (!h.ok) {
      setGalatPanel(h);
      return;
    }
    setDetail(h.data);
  };

  const tampilkanFoto = async () => {
    if (bukaId === null) return;
    setSibuk(true);
    const h = await api.fotoUrl(bukaId);
    setSibuk(false);
    if (!h.ok) {
      setGalatPanel(h);
      return;
    }
    setGalatPanel(null);
    setFoto(h.data);
  };

  /** Galat keputusan: kode 409 punya perilaku khusus; sisanya tampil sebagai pesan di panel. */
  const tanganiGalat = async (g: Gagal) => {
    setGalatPanel(g);
    if (g.kode === KODE_PEMISAHAN_TUGAS) setTerkunci(true);
    if (g.kode === KODE_NIK_TERPAKAI) setKodeAlasan(KODE_NIK_TIDAK_TERSEDIA);
    if (g.kode === KODE_SUDAH_DIPUTUSKAN) {
      setTerkunci(true);
      await muat();
    }
  };

  const setujui = async () => {
    if (bukaId === null) return;
    setSibuk(true);
    const h = await api.setujui(bukaId);
    setSibuk(false);
    if (!h.ok) {
      await tanganiGalat(h);
      return;
    }
    tutupPanel();
    setInfo("Pengajuan disetujui. Pasien diberi tahu.");
    await muat();
  };

  const tolak = async () => {
    if (bukaId === null) return;
    if (kodeAlasan === "") {
      setGalatIsian("Pilih alasan untuk menolak.");
      return;
    }
    if (kodeAlasan === KODE_LAINNYA && keterangan.trim() === "") {
      setGalatIsian("Keterangan wajib diisi untuk alasan Lainnya.");
      return;
    }
    setGalatIsian(null);
    setSibuk(true);
    const h = await api.tolak(bukaId, kodeAlasan, keterangan);
    setSibuk(false);
    if (!h.ok) {
      await tanganiGalat(h);
      return;
    }
    tutupPanel();
    setInfo("Pengajuan ditolak. Alasan tampil ke pasien dan pasien dapat mengunggah ulang.");
    await muat();
  };

  const bisaMemutuskan = detail !== null && detail.can_decide && !terkunci;
  const pemisahanTugas = galatPanel?.kode === KODE_PEMISAHAN_TUGAS;
  const sudahDiputuskan = detail !== null && !detail.can_decide;

  return (
    <>
      <h1>Antrean verifikasi identitas</h1>
      <p className="bo-bantu">
        Urut dari pengajuan terlama. NIK di daftar termasking; NIK lengkap hanya di panel detail dan pembukaannya dicatat. Semua
        waktu dalam WIB.
      </p>
      {info === null ? null : <Alert variant="success" title={info} />}
      {galat === null ? null : (
        <Alert variant="danger" title="Antrean gagal dimuat">
          {galat.pesan}
        </Alert>
      )}
      <div className="bo-verif">
        <section aria-label="Antrean">
          {item === null && galat === null ? (
            <Card loading aria-label="Memuat antrean" />
          ) : item !== null && item.length === 0 ? (
            <Card>
              <h2>Antrean kosong</h2>
              <p>Tidak ada pengajuan yang menunggu verifikasi.</p>
            </Card>
          ) : item === null ? null : (
            <div className="bo-tabel-bungkus">
              <table className="bo-tabel">
                <caption className="bo-sr">Antrean verifikasi identitas</caption>
                <thead>
                  <tr>
                    <th scope="col">Diajukan (WIB)</th>
                    <th scope="col">Nama</th>
                    <th scope="col">Peran</th>
                    <th scope="col">Dokumen</th>
                    <th scope="col">NIK</th>
                    <th scope="col">Aksi</th>
                  </tr>
                </thead>
                <tbody>
                  {item.map((k) => (
                    <tr key={k.identity_verification_request_id} aria-selected={k.identity_verification_request_id === bukaId}>
                      <td>{formatWib(k.submitted_at)}</td>
                      <th scope="row">{k.full_name}</th>
                      <td>{k.is_account_owner ? "Pemilik akun" : "Anggota keluarga"}</td>
                      <td>{dokumen(k.document_type)}</td>
                      <td>{nikTermasking(k.nik_last4)}</td>
                      <td>
                        <Button
                          variant="secondary"
                          disabled={sibuk}
                          onClick={() => void buka(k.identity_verification_request_id)}
                        >
                          Buka
                          <span className="bo-sr"> pengajuan {k.full_name}</span>
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {berikut === null ? null : (
                <Button variant="secondary" onClick={() => void muat(berikut)}>
                  Muat lebih banyak
                </Button>
              )}
            </div>
          )}
        </section>

        {bukaId === null ? null : (
          <section aria-label="Detail pengajuan" className="bo-verif-detail">
            {detail === null && galatPanel === null ? <Card loading aria-label="Memuat detail" /> : null}
            {detail === null ? null : (
              <>
                <div className="bo-judul-baris">
                  <h2>
                    {detail.full_name} &middot; {detail.is_account_owner ? "pemilik akun" : "anggota keluarga"} &middot;{" "}
                    {dokumen(detail.document_type)}
                  </h2>
                  <Badge tone={detail.status === "approved" ? "success" : detail.status === "pending" ? "info" : "neutral"}>
                    {LABEL_STATUS[detail.status]}
                  </Badge>
                </div>
                <dl className="bo-dl">
                  <dt>Tanggal lahir</dt>
                  <dd>{formatTanggal(detail.birth_date)}</dd>
                  <dt>Jenis kelamin</dt>
                  <dd>{LABEL_SEX[detail.sex] ?? detail.sex}</dd>
                  <dt>NIK</dt>
                  <dd data-testid="nik-lengkap">{detail.nik} (lengkap untuk Verifikator)</dd>
                  <dt>Diajukan</dt>
                  <dd>{formatWib(detail.submitted_at)}</dd>
                  {detail.decided_at === null || detail.decided_at === undefined ? null : (
                    <>
                      <dt>Diputuskan</dt>
                      <dd>{formatWib(detail.decided_at)}</dd>
                    </>
                  )}
                </dl>
                {detail.document_type === "kk" ? (
                  <p className="bo-bantu">Pastikan pemilik akun tercantum sebagai orang tua atau wali di KK. Kalau tidak tercantum, tolak dengan alasan itu.</p>
                ) : null}
                <h3>Foto {dokumen(detail.document_type)} (dibuka = dicatat)</h3>
                {foto === null ? (
                  <Button variant="secondary" loading={sibuk} onClick={() => void tampilkanFoto()}>
                    Tampilkan foto dokumen
                  </Button>
                ) : (
                  <FotoDokumenView foto={foto} jenis={detail.document_type} onKedaluwarsa={() => { setFoto(null); }} />
                )}
              </>
            )}
            {galatPanel === null ? null : (
              <Alert variant={galatPanel.kode === KODE_NIK_TERPAKAI || galatPanel.kode === KODE_SUDAH_DIPUTUSKAN ? "warning" : "danger"} title={
                galatPanel.kode === KODE_SUDAH_DIPUTUSKAN
                  ? "Pengajuan sudah diputuskan"
                  : galatPanel.kode === KODE_NIK_TERPAKAI
                    ? "NIK sudah dipakai"
                    : pemisahanTugas
                      ? "Tidak dapat diputuskan"
                      : "Tindakan gagal"
              }>
                {galatPanel.pesan}
              </Alert>
            )}
            {detail === null ? null : sudahDiputuskan ? (
              <p className="bo-bantu">Pengajuan ini tidak menunggu keputusan lagi.</p>
            ) : bisaMemutuskan ? (
              <div className="bo-form">
                <div className="rs-field">
                  <label className="rs-field__label" htmlFor={`${uid}-alasan`}>
                    Alasan penolakan
                  </label>
                  <select
                    id={`${uid}-alasan`}
                    className="rs-field__input"
                    value={kodeAlasan}
                    onChange={(e) => { setKodeAlasan(e.target.value); setGalatIsian(null); }}
                  >
                    <option value="">Pilih alasan</option>
                    {alasan.map((a) => (
                      <option key={a.code} value={a.code}>
                        {a.label}
                      </option>
                    ))}
                  </select>
                </div>
                <div className="rs-field">
                  <label className="rs-field__label" htmlFor={`${uid}-ket`}>
                    Keterangan tambahan (opsional, tampil ke pasien; wajib untuk Lainnya)
                  </label>
                  <textarea
                    id={`${uid}-ket`}
                    className="rs-field__input bo-textarea"
                    rows={3}
                    maxLength={BATAS_KETERANGAN}
                    value={keterangan}
                    onChange={(e) => { setKeterangan(e.target.value); setGalatIsian(null); }}
                  />
                </div>
                {galatIsian === null ? null : (
                  <p role="alert" className="rs-field__error">
                    <span aria-hidden="true" className="rs-field__error-icon">!</span>
                    <span>{galatIsian}</span>
                  </p>
                )}
                <div className="bo-aksi">
                  <Button variant="destructive" loading={sibuk} onClick={() => void tolak()}>
                    Tolak
                  </Button>
                  <Button
                    variant="primary"
                    loading={sibuk}
                    disabled={galatPanel?.kode === KODE_NIK_TERPAKAI}
                    onClick={() => void setujui()}
                  >
                    Setujui
                  </Button>
                  <Button variant="text" onClick={tutupPanel}>
                    Tutup
                  </Button>
                </div>
              </div>
            ) : (
              <div className="bo-aksi">
                <Button variant="text" onClick={tutupPanel}>
                  Tutup
                </Button>
              </div>
            )}
            {detail === null && galatPanel !== null ? (
              <div className="bo-aksi">
                <Button variant="text" onClick={tutupPanel}>
                  Tutup
                </Button>
              </div>
            ) : null}
          </section>
        )}
      </div>
    </>
  );
}
