"use client";

import { Alert, Badge, Button, Card } from "@rs/ui";
import { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";
import { buatKlienPerusahaan, KODE_CSRF_DITOLAK, type Gagal, type KlienPerusahaan } from "../../lib/perusahaan/klien.ts";
import {
  LABEL_DOKUMEN_PERUSAHAAN,
  LABEL_ORIGIN,
  LABEL_FIELD_KOREKSI,
  LABEL_STATUS_PERUSAHAAN,
  nibTermasking,
  nilaiSebelumnya,
  sumber,
  TEKS_DATA_DIKOREKSI,
  TEKS_NIB_DIPAKAI,
  type AlasanTolakPerusahaan,
  type DetailPerusahaan,
  type DokumenUrl,
  type FieldKoreksi,
  type ItemAntreanPerusahaan,
  type JenisDokumen,
} from "../../lib/perusahaan/model.ts";
import { FormBuatPerusahaan } from "./FormBuatPerusahaan.tsx";
import { formatWib } from "../../lib/verifikasi/model.ts";

export interface PanelPerusahaanProps {
  readonly csrfToken: string;
  /** Hanya untuk test. */
  readonly klien?: KlienPerusahaan;
}

const KODE_SUDAH_DIPUTUSKAN = "4090001";
const KODE_DOKUMEN_BERUBAH = "4090114";

/**
 * Dokumen perusahaan (ADR-018 butir 3, US-1902 AC1). Hanya dari URL unduh bertanda tangan berumur pendek yang
 * diminta eksplisit (E-05 tercatat sebelum URL terbit). URL hanya di memori komponen: tidak ditulis ke storage,
 * tidak dicatat, `referrerPolicy="no-referrer"`, dan dilepas saat kedaluwarsa atau panel ditutup. Gambar tampil
 * inline; tipe lain (misalnya PDF) hanya lewat tautan tab baru tanpa referrer.
 */
function DokumenView({ dok, jenis, onKedaluwarsa }: { dok: DokumenUrl; jenis: JenisDokumen; onKedaluwarsa: () => void }) {
  const [gagalMuat, setGagalMuat] = useState(false);
  const kedaluwarsa = useRef(onKedaluwarsa);
  kedaluwarsa.current = onKedaluwarsa;
  useEffect(() => {
    const sisa = new Date(dok.expires_at).getTime() - Date.now();
    if (Number.isNaN(sisa) || sisa <= 0) {
      kedaluwarsa.current();
      return;
    }
    const t = setTimeout(() => { kedaluwarsa.current(); }, sisa);
    return () => { clearTimeout(t); };
  }, [dok.expires_at]);
  if (gagalMuat) {
    return (
      <Alert variant="warning" title="Dokumen tidak dapat dibuka">
        Tautan dokumen kedaluwarsa atau gagal dibuka. Klik Buka lagi untuk membuat tautan baru (dicatat E-05).
      </Alert>
    );
  }
  if (dok.mime_type.startsWith("image/")) {
    return (
      <img
        className="bo-foto"
        src={dok.url}
        alt={`${LABEL_DOKUMEN_PERUSAHAAN[jenis]} pengajuan`}
        referrerPolicy="no-referrer"
        onError={() => { setGagalMuat(true); }}
      />
    );
  }
  return (
    <a href={dok.url} target="_blank" rel="noopener noreferrer nofollow" referrerPolicy="no-referrer">
      Buka {LABEL_DOKUMEN_PERUSAHAAN[jenis]} di tab baru
    </a>
  );
}

/**
 * Penanda "Data dikoreksi" di samping field yang dikoreksi HR pada pengajuan ulang ini, beserta nilai putaran
 * sebelumnya (CR-040, keputusan user Q34 = A). Tidak tampil bila field tidak berubah. Status tidak hanya lewat
 * warna: badge bersimbol dan bertulisan (DG §5.6).
 */
function PenandaKoreksi({ detail, field }: { detail: DetailPerusahaan; field: FieldKoreksi }) {
  const sebelumnya = nilaiSebelumnya(detail, field);
  if (sebelumnya === undefined) return null;
  return (
    <span className="bo-koreksi" data-testid={`koreksi-${field}`}>
      <Badge tone="warning">{TEKS_DATA_DIKOREKSI}</Badge>{" "}
      <span className="bo-koreksi__lama">Sebelumnya: {sebelumnya}</span>
    </span>
  );
}

/** Layar antrean verifikasi perusahaan (wireframe C13 bagian 1, US-1902): terlama dulu, panel detail dan dokumen. */
export function PanelPerusahaan({ csrfToken, klien }: PanelPerusahaanProps) {
  const api = useMemo(() => klien ?? buatKlienPerusahaan(csrfToken), [klien, csrfToken]);
  const [item, setItem] = useState<readonly ItemAntreanPerusahaan[] | null>(null);
  const [berikut, setBerikut] = useState<string | null>(null);
  const [alasan, setAlasan] = useState<readonly AlasanTolakPerusahaan[]>([]);
  const [galat, setGalat] = useState<Gagal | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [bukaId, setBukaId] = useState<string | null>(null);
  const [detail, setDetail] = useState<DetailPerusahaan | null>(null);
  const [galatPanel, setGalatPanel] = useState<Gagal | null>(null);
  const [dokumenAktif, setDokumenAktif] = useState<{ jenis: JenisDokumen; dok: DokumenUrl } | null>(null);
  const [sibuk, setSibuk] = useState(false);
  const [kodeAlasan, setKodeAlasan] = useState("");
  const [galatIsian, setGalatIsian] = useState<string | null>(null);
  const [terkunci, setTerkunci] = useState(false);
  const [formBuat, setFormBuat] = useState(false);
  const uid = useId();

  const tutupPanel = useCallback(() => {
    // Data sensitif (NIB/NPWP lengkap, URL dokumen) dibuang dari memori saat panel ditutup.
    setBukaId(null);
    setDetail(null);
    setDokumenAktif(null);
    setGalatPanel(null);
    setKodeAlasan("");
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

  if (galat?.status === 403 && galat.kode !== KODE_CSRF_DITOLAK) {
    return (
      <>
        <h1>Antrean verifikasi perusahaan</h1>
        <Alert variant="warning" title="Tidak ada akses">
          Akun Anda tidak memiliki sub-peran Verifikator untuk memeriksa dokumen perusahaan.
        </Alert>
      </>
    );
  }

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

  const bukaDokumen = async (jenis: JenisDokumen) => {
    if (bukaId === null) return;
    setSibuk(true);
    const h = await api.dokumenUrl(bukaId, jenis);
    setSibuk(false);
    if (!h.ok) {
      setGalatPanel(h);
      return;
    }
    setGalatPanel(null);
    setDokumenAktif({ jenis, dok: h.data });
  };

  /** Keputusan pertama berlaku (AC4): 409 mengunci panel dan menyegarkan antrean. */
  const tanganiGalat = async (g: Gagal) => {
    setGalatPanel(g);
    if (g.kode === KODE_SUDAH_DIPUTUSKAN) {
      setTerkunci(true);
      await muat();
    }
    if (g.kode === KODE_DOKUMEN_BERUBAH && bukaId !== null) {
      // Pengajuan berubah sejak detail dibuka (dokumen diganti, atau data dikoreksi HR dan diajukan ulang, CR-036):
      // muat ulang detail (token versi baru) agar keputusan berikutnya sah.
      // Dokumen yang sedang tampil bisa sudah tidak berlaku: ditutup, harus dibuka ulang dari detail baru.
      setDokumenAktif(null);
      const baru = await api.detail(bukaId);
      if (baru.ok) setDetail(baru.data);
    }
  };

  const setujui = async () => {
    if (bukaId === null || detail === null) return;
    setSibuk(true);
    const h = await api.setujui(bukaId, detail.documents_version);
    setSibuk(false);
    if (!h.ok) {
      await tanganiGalat(h);
      return;
    }
    tutupPanel();
    setInfo("Perusahaan disetujui dan berstatus Aktif.");
    await muat();
  };

  const tolak = async () => {
    if (bukaId === null || detail === null) return;
    if (kodeAlasan === "") {
      setGalatIsian("Pilih alasan untuk menolak.");
      return;
    }
    setGalatIsian(null);
    setSibuk(true);
    const h = await api.tolak(bukaId, kodeAlasan, detail.documents_version);
    setSibuk(false);
    if (!h.ok) {
      await tanganiGalat(h);
      return;
    }
    tutupPanel();
    setInfo("Pengajuan ditolak. HR melihat alasannya dan dapat memperbaiki lalu mengajukan ulang.");
    await muat();
  };

  const bisaMemutuskan = detail !== null && detail.can_decide && !terkunci;
  const sudahDiputuskan = detail !== null && !detail.can_decide;
  const konflik = galatPanel?.kode === KODE_SUDAH_DIPUTUSKAN;

  return (
    <>
      <h1>Antrean verifikasi perusahaan</h1>
      <p className="bo-bantu">
        Urut dari pengajuan terlama. NIB di daftar termasking; NIB, NPWP, dan dokumen hanya di panel detail dan setiap
        pembukaan dokumen dicatat. Semua waktu dalam WIB.
      </p>
      <p className="bo-bantu" data-testid="belum-tersedia">
        Daftar perusahaan dan anggota HR belum tersedia (menunggu endpoint backoffice untuk daftar).
      </p>
      {formBuat ? (
        <Card>
          <FormBuatPerusahaan
            api={api}
            onGalatAkses={(g) => {
              setFormBuat(false);
              setItem(null);
              setGalat(g);
            }}
            onBatal={() => {
              setFormBuat(false);
            }}
            onSelesai={() => {
              setFormBuat(false);
              tutupPanel();
              setInfo("Perusahaan dibuat dan berstatus Aktif. Undangan aktivasi untuk HR pertama sedang diproses.");
              void muat();
            }}
          />
        </Card>
      ) : (
        <div className="bo-aksi">
          <Button
            variant="primary"
            onClick={() => {
              setInfo(null);
              setFormBuat(true);
            }}
          >
            Buat perusahaan
          </Button>
        </div>
      )}
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
              <p>Tidak ada perusahaan yang menunggu verifikasi.</p>
            </Card>
          ) : item === null ? null : (
            <div className="bo-tabel-bungkus">
              <table className="bo-tabel">
                <caption className="bo-sr">Antrean verifikasi perusahaan</caption>
                <thead>
                  <tr>
                    <th scope="col">Diajukan (WIB)</th>
                    <th scope="col">Badan usaha</th>
                    <th scope="col">NIB</th>
                    <th scope="col">Sumber</th>
                    <th scope="col">Aksi</th>
                  </tr>
                </thead>
                <tbody>
                  {item.map((k) => (
                    <tr key={k.corporate_client_id} aria-selected={k.corporate_client_id === bukaId}>
                      <td>{formatWib(k.submitted_at)}</td>
                      <th scope="row">{k.legal_name}</th>
                      <td>
                        {nibTermasking(k.nib)}
                        {k.nib_used_by_other_company === true ? (
                          <>
                            {" "}
                            <Badge tone="warning">{TEKS_NIB_DIPAKAI}</Badge>
                          </>
                        ) : null}
                      </td>
                      <td>{sumber(k)}</td>
                      <td>
                        <Button variant="secondary" disabled={sibuk} onClick={() => void buka(k.corporate_client_id)}>
                          Buka
                          <span className="bo-sr"> pengajuan {k.legal_name}</span>
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
          <section aria-label="Detail perusahaan" className="bo-verif-detail">
            {detail === null && galatPanel === null ? <Card loading aria-label="Memuat detail" /> : null}
            {detail === null ? null : (
              <>
                <div className="bo-judul-baris">
                  <h2>{detail.legal_name}</h2>
                  <Badge tone={detail.status === "active" ? "success" : detail.status === "pending_verification" ? "info" : "neutral"}>
                    {LABEL_STATUS_PERUSAHAAN[detail.status]}
                  </Badge>
                </div>
                {detail.nib_used_by_other_company ? (
                  <Alert variant="warning" title={TEKS_NIB_DIPAKAI}>
                    NIB ini juga tercatat pada perusahaan lain. Pastikan pendaftar berhak atas NIB ini. Pengajuan tidak
                    dapat disetujui bila NIB sudah dipakai perusahaan Aktif lain.
                  </Alert>
                ) : null}
                {detail.corrected_fields.length === 0 ? null : (
                  <Alert variant="info" title="HR mengoreksi data pada pengajuan ulang ini">
                    Periksa ulang yang bertanda {TEKS_DATA_DIKOREKSI}:{" "}
                    {detail.corrected_fields.map((f) => LABEL_FIELD_KOREKSI[f.field]).join(", ")}.
                  </Alert>
                )}
                <dl className="bo-dl">
                  {nilaiSebelumnya(detail, "legal_name") === undefined ? null : (
                    <>
                      <dt>Nama badan usaha</dt>
                      <dd>
                        {detail.legal_name} <PenandaKoreksi detail={detail} field="legal_name" />
                      </dd>
                    </>
                  )}
                  <dt>NIB</dt>
                  <dd>
                    <span data-testid="nib-lengkap">{detail.nib} (lengkap untuk Verifikator)</span>{" "}
                    <PenandaKoreksi detail={detail} field="nib" />
                  </dd>
                  <dt>NPWP</dt>
                  <dd>
                    <span data-testid="npwp-lengkap">{detail.npwp}</span> <PenandaKoreksi detail={detail} field="npwp" />
                  </dd>
                  <dt>Alamat kantor</dt>
                  <dd>
                    {detail.office_address} <PenandaKoreksi detail={detail} field="office_address" />
                  </dd>
                  <dt>Telepon kantor</dt>
                  <dd>
                    {detail.office_phone} <PenandaKoreksi detail={detail} field="office_phone" />
                  </dd>
                  <dt>Sumber</dt>
                  <dd>
                    {LABEL_ORIGIN[detail.origin]} (putaran {detail.submission_round})
                  </dd>
                  <dt>Diajukan</dt>
                  <dd>{formatWib(detail.submitted_at)}</dd>
                  {detail.decided_at === null || detail.decided_at === undefined ? null : (
                    <>
                      <dt>Diputuskan</dt>
                      <dd>{formatWib(detail.decided_at)}</dd>
                    </>
                  )}
                </dl>
                <p className="bo-bantu">
                  Periksa bukti kewenangan: surat kuasa atau surat penunjukan dari perusahaan, ditandatangani pejabat
                  berwenang, nama badan usaha cocok dengan NIB, dan nama pendaftar tercantum. Bila tidak memenuhi, tolak
                  dengan alasan yang sesuai.
                </p>
                <h3>Dokumen (dibuka = dicatat)</h3>
                <ul className="bo-dokumen">
                  {detail.documents.map((jenis) => (
                    <li key={jenis}>
                      <span>{LABEL_DOKUMEN_PERUSAHAAN[jenis]}</span>{" "}
                      <Button variant="secondary" disabled={sibuk} onClick={() => void bukaDokumen(jenis)}>
                        Buka
                        <span className="bo-sr"> {LABEL_DOKUMEN_PERUSAHAAN[jenis]}</span>
                      </Button>
                    </li>
                  ))}
                </ul>
                {dokumenAktif === null ? null : (
                  <DokumenView
                    dok={dokumenAktif.dok}
                    jenis={dokumenAktif.jenis}
                    onKedaluwarsa={() => { setDokumenAktif(null); }}
                  />
                )}
              </>
            )}
            {galatPanel === null ? null : (
              <Alert variant={konflik ? "warning" : "danger"} title={konflik ? "Pengajuan sudah diputuskan" : "Tindakan gagal"}>
                {galatPanel.pesan}
              </Alert>
            )}
            {detail === null ? null : sudahDiputuskan ? (
              <>
                <p className="bo-bantu">Pengajuan ini tidak menunggu keputusan lagi.</p>
                <div className="bo-aksi">
                  <Button variant="text" onClick={tutupPanel}>
                    Tutup
                  </Button>
                </div>
              </>
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
                  <Button variant="primary" loading={sibuk} onClick={() => void setujui()}>
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
