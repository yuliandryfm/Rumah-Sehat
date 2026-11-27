"use client";

import { Alert, Badge, Button, Checkbox, Skeleton, TextField } from "@rs/ui";
import Link from "next/link";
import { useCallback, useEffect, useId, useMemo, useRef, useState, type ChangeEvent, type Ref } from "react";
import type { Gagal } from "../../lib/hr/klien.ts";
import {
  FIELD_KOREKSI,
  LABEL_KOREKSI,
  MAKS_KOREKSI,
  PETUNJUK_KOREKSI,
  badanKoreksi,
  galatServer,
  isianAwal,
  periksaKoreksi,
  type FieldKoreksi,
  type GalatKoreksi,
  type IsianKoreksi,
  type UbahTersamar,
} from "../../lib/verifikasi/koreksi.ts";
import {
  buatKlienVerifikasi,
  type JenisDokumen,
  type GagalKoreksi,
  type KlienVerifikasi,
  type Muatan,
  type Ringkasan,
  type StatusDokumen,
  type UrlUnduh,
} from "../../lib/verifikasi/klien.ts";
import {
  ACCEPT,
  DOKUMEN,
  LABEL_STATUS,
  PETUNJUK_BERKAS,
  formatUkuran,
  formatWib,
  labelAlasan,
  labelDokumen,
  periksaBerkas,
  termasking,
  urlUnduhAman,
} from "../../lib/verifikasi/model.ts";
import { GalatAkses, adalahGalatAkses } from "../hr/GalatAkses.tsx";

export interface StatusVerifikasiProps {
  readonly csrfToken: string;
  /** Hanya untuk test. */
  readonly klien?: KlienVerifikasi;
  readonly arahkan?: (url: string) => void;
}

/**
 * Status verifikasi perusahaan (F1-T103; US-1901 AC2, AC6; US-1902 AC3; wireframe K01 layar 7-8; brief KR-09,
 * KR-10): status (Menunggu verifikasi / Ditolak dengan alasan / Aktif), tiga dokumen wajib dengan status,
 * unggah dan ganti lewat slot, unduh lewat URL berumur pendek, dan "Ajukan ulang" saat Ditolak. Saat Ditolak,
 * HR dapat mengoreksi data perusahaan di bagian "Ajukan ulang" (CR-036, Q20 = A, KEPUTUSAN-413): koreksi dikirim
 * bersama pengajuan ulang dalam satu aksi; NIB/NPWP tetap tersamar dan hanya diketik ulang bila dipilih diubah.
 * Status "Aktif" berasal dari 409 `4090112` (endpoint dokumen hanya melayani perusahaan yang belum Aktif).
 * Semua keputusan di server; layar ini tidak menyimpan berkas atau isian di perangkat (DG D-K01).
 */
export function StatusVerifikasi({ csrfToken, klien, arahkan }: StatusVerifikasiProps) {
  const api = useMemo(() => klien ?? buatKlienVerifikasi(csrfToken), [klien, csrfToken]);
  const [muatan, setMuatan] = useState<Muatan | null>(null);
  const [galat, setGalat] = useState<Gagal | null>(null);
  const [galatAksi, setGalatAksi] = useState<Gagal | null>(null);
  const [galatAjukan, setGalatAjukan] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [ajukanSibuk, setAjukanSibuk] = useState(false);
  const [memuat, setMemuat] = useState(false);
  // CR-036: isian koreksi data perusahaan (hanya di memori; diisi dari profil saat status Ditolak dimuat).
  const [isian, setIsian] = useState<IsianKoreksi | null>(null);
  const [ubah, setUbah] = useState<UbahTersamar>({ nib: false, npwp: false });
  const [galatField, setGalatField] = useState<GalatKoreksi>({});
  const formKoreksi = useRef<HTMLFieldSetElement>(null);
  const fokusKe = useRef<FieldKoreksi | null>(null);

  const muat = useCallback(async () => {
    setMemuat(true);
    const h = await api.muat();
    setMemuat(false);
    if (h.ok) {
      setMuatan(h.data);
      setGalat(null);
      if (h.data.jenis === "dokumen") {
        setIsian(isianAwal(h.data.ringkasan.company));
        setUbah({ nib: false, npwp: false });
        setGalatField({});
      }
    } else {
      setGalat(h);
    }
  }, [api]);

  // Fokus ke isian galat pertama setelah render (DG §5.3, A12).
  useEffect(() => {
    const f = fokusKe.current;
    fokusKe.current = null;
    if (f !== null) formKoreksi.current?.querySelector<HTMLInputElement>(`[name="koreksi-${f}"]`)?.focus();
  });

  useEffect(() => {
    void muat();
  }, [muat]);

  const terima = useCallback((r: Ringkasan) => {
    setMuatan({ jenis: "dokumen", ringkasan: r });
  }, []);

  if (adalahGalatAkses(galat)) return <GalatAkses galat={galat} {...(arahkan === undefined ? {} : { arahkan })} />;
  // 401 saat aksi (sesi berakhir atau akun dinonaktifkan di tengah jalan) sama dengan saat memuat.
  if (adalahGalatAkses(galatAksi)) return <GalatAkses galat={galatAksi} {...(arahkan === undefined ? {} : { arahkan })} />;

  const ajukanUlang = async (r: Ringkasan) => {
    setInfo(null);
    setGalatAjukan(null);
    if (!r.complete) {
      // Bantuan klien; server tetap menolak 4090115 bila dokumen belum lengkap.
      setGalatAjukan("Dokumen belum lengkap. Unggah ketiga dokumen terlebih dahulu, lalu ajukan ulang.");
      return;
    }
    // CR-036: koreksi data (bila ada) dikirim bersama pengajuan ulang dalam satu aksi.
    const badan = isian === null ? {} : badanKoreksi(isian, ubah, r.company);
    const adaKoreksi = Object.keys(badan).length > 0;
    if (adaKoreksi && isian !== null) {
      const g = periksaKoreksi(isian, ubah, r.company);
      setGalatField(g);
      const pertama = FIELD_KOREKSI.find((f) => g[f] !== undefined);
      if (pertama !== undefined) {
        fokusKe.current = pertama;
        setGalatAjukan("Ada data perusahaan yang perlu diperbaiki. Periksa pesan di setiap isian.");
        return;
      }
    } else {
      setGalatField({});
    }
    setAjukanSibuk(true);
    const h = adaKoreksi ? await api.koreksiDanAjukanUlang(badan) : await api.ajukanUlang();
    setAjukanSibuk(false);
    if (h.ok) {
      terima(h.data);
      setInfo(
        adaKoreksi
          ? "Data perusahaan dikoreksi dan pengajuan ulang terkirim. Status perusahaan kembali Menunggu verifikasi."
          : "Pengajuan ulang terkirim. Status perusahaan kembali Menunggu verifikasi.",
      );
    } else if (adalahGalatAkses(h)) {
      setGalatAksi(h);
    } else {
      if (h.status === 422 && adaKoreksi) {
        const g = galatServer((h as GagalKoreksi).isian);
        setGalatField(g);
        fokusKe.current = FIELD_KOREKSI.find((f) => g[f] !== undefined) ?? null;
      }
      setGalatAjukan(h.pesan);
    }
  };

  return (
    <section aria-labelledby="judul-verifikasi" className="kp-bagian kp-lebar">
      <h1 id="judul-verifikasi">Status verifikasi perusahaan</h1>
      <div role="status" aria-live="polite" className="kp-umum">
        {info}
      </div>
      {galat === null ? null : (
        <Alert variant="danger" title="Status verifikasi tidak dapat dimuat">
          <p>{galat.pesan}</p>
          <Button variant="secondary" loading={memuat} loadingLabel="Memuat..." onClick={() => void muat()}>
            Coba lagi
          </Button>
        </Alert>
      )}
      {muatan === null && galat === null ? <Skeleton lines={4} /> : null}
      {muatan?.jenis === "aktif" ? <Aktif /> : null}
      {muatan?.jenis === "dokumen" ? (
        <>
          <RingkasanStatus r={muatan.ringkasan} />
          <section aria-labelledby="judul-dokumen" className="kp-bagian">
            <h2 id="judul-dokumen">Dokumen perusahaan</h2>
            <p className="kp-kecil" id="petunjuk-berkas">
              {PETUNJUK_BERKAS}
            </p>
            {DOKUMEN.map((d) => {
              const status = muatan.ringkasan.documents.find((x) => x.kind === d.jenis) ?? { kind: d.jenis, uploaded: false };
              return (
                <BarisDokumen
                  key={d.jenis}
                  jenis={d.jenis}
                  status={status}
                  api={api}
                  onTerima={(r) => {
                    terima(r);
                    setGalatAjukan(null);
                    setInfo(`${labelDokumen(d.jenis)} berhasil diunggah.`);
                  }}
                  onGalatAkses={setGalatAksi}
                />
              );
            })}
          </section>
          {muatan.ringkasan.status === "rejected" ? (
            <section aria-labelledby="judul-ajukan" className="kp-bagian">
              <h2 id="judul-ajukan">Ajukan ulang</h2>
              <p>
                Setelah dokumen yang perlu diperbaiki diganti dan data perusahaan yang salah dikoreksi, ajukan ulang
                supaya Verifikator memeriksa kembali.
              </p>
              {isian === null ? null : (
                <FormKoreksi
                  ref={formKoreksi}
                  profil={muatan.ringkasan.company}
                  isian={isian}
                  ubah={ubah}
                  galat={galatField}
                  onIsian={(f, v) => {
                    setIsian((s) => (s === null ? s : { ...s, [f]: v }));
                  }}
                  onUbah={(f, on) => {
                    setUbah((s) => ({ ...s, [f]: on }));
                    if (!on) {
                      setIsian((s) => (s === null ? s : { ...s, [f]: "" }));
                      setGalatField((g) => Object.fromEntries(Object.entries(g).filter(([k]) => k !== f)));
                    }
                  }}
                />
              )}
              <div>
                <Button
                  loading={ajukanSibuk}
                  loadingLabel="Mengajukan..."
                  aria-describedby={galatAjukan === null ? undefined : "galat-ajukan"}
                  onClick={() => void ajukanUlang(muatan.ringkasan)}
                >
                  Ajukan ulang
                </Button>
              </div>
              {galatAjukan === null ? null : (
                <p id="galat-ajukan" className="rs-field__error" role="alert">
                  <span aria-hidden="true" className="rs-field__error-icon">
                    !
                  </span>
                  <span>{galatAjukan}</span>
                </p>
              )}
            </section>
          ) : null}
          <Profil r={muatan.ringkasan} />
        </>
      ) : null}
    </section>
  );
}

function Aktif() {
  const s = LABEL_STATUS.active;
  return (
    <>
      <p>
        <Badge tone={s.tone}>{s.label}</Badge>
      </p>
      <Alert variant="success" title="Perusahaan Anda sudah terverifikasi">
        Dokumen perusahaan tidak perlu diunggah lagi. <Link href="/" prefetch={false}>Buka dasbor</Link>
      </Alert>
    </>
  );
}

const LANGKAH = ["Pendaftaran terkirim", "Pemeriksaan dokumen oleh Verifikator", "Keputusan"] as const;

function RingkasanStatus({ r }: { r: Ringkasan }) {
  const s = LABEL_STATUS[r.status];
  const sekarang = r.status === "rejected" ? 2 : r.complete ? 1 : 0;
  return (
    <>
      <p className="kp-baris-status">
        <Badge tone={s.tone}>{s.label}</Badge>
        <span className="kp-kecil">Diajukan {formatWib(r.company.submitted_at)}</span>
      </p>
      {r.status === "rejected" ? (
        <Alert variant="danger" title="Pengajuan ditolak">
          <p>
            Alasan: <span data-testid="alasan-tolak">{labelAlasan(r.rejection_reason)}</span>.
          </p>
          <p>Ganti dokumen yang perlu diperbaiki di bawah, lalu tekan Ajukan ulang.</p>
        </Alert>
      ) : r.complete ? (
        <Alert variant="info" title="Dokumen sedang diperiksa">
          Tim Ruang Sehat sedang memeriksa dokumen perusahaan Anda. Layanan pesanan terbuka setelah perusahaan
          disetujui. Pemberitahuan lewat email belum tersedia, jadi periksa halaman ini secara berkala.
        </Alert>
      ) : (
        <Alert variant="warning" title="Lengkapi dokumen perusahaan">
          Pengajuan baru diperiksa Verifikator setelah Dokumen NIB, Dokumen NPWP, dan Bukti kewenangan pendaftar
          diunggah.
        </Alert>
      )}
      <ol className="kp-linimasa" aria-label="Tahapan verifikasi">
        {LANGKAH.map((l, i) => (
          <li key={l} aria-current={i === sekarang ? "step" : undefined} data-selesai={i < sekarang ? "true" : undefined}>
            {l}
            {i === 2 && r.status === "rejected" ? ": Ditolak" : ""}
          </li>
        ))}
      </ol>
    </>
  );
}

function Profil({ r }: { r: Ringkasan }) {
  const p = r.company;
  return (
    <section aria-labelledby="judul-profil" className="kp-bagian">
      <h2 id="judul-profil">Profil perusahaan</h2>
      <dl className="kp-profil">
        <dt>Nama badan usaha</dt>
        <dd>{p.legal_name}</dd>
        <dt>NIB</dt>
        <dd>{termasking(p.nib)}</dd>
        <dt>NPWP</dt>
        <dd>{termasking(p.npwp)}</dd>
        <dt>Alamat kantor</dt>
        <dd>{p.office_address}</dd>
        <dt>Telepon kantor</dt>
        <dd>{p.office_phone}</dd>
      </dl>
      {/* CR-036 menggantikan teks sementara Q31 ("hubungi Layanan Pelanggan"): koreksi hanya saat Ditolak (Q20 = A). */}
      <p className="kp-kecil">
        {r.status === "rejected"
          ? "Bila ada data yang salah, koreksi di bagian Ajukan ulang di atas, lalu ajukan ulang."
          : "Data perusahaan dapat dikoreksi bila pengajuan ditolak Verifikator."}
      </p>
    </section>
  );
}

interface FormKoreksiProps {
  readonly ref: Ref<HTMLFieldSetElement>;
  readonly profil: Ringkasan["company"];
  readonly isian: IsianKoreksi;
  readonly ubah: UbahTersamar;
  readonly galat: GalatKoreksi;
  readonly onIsian: (f: FieldKoreksi, nilai: string) => void;
  readonly onUbah: (f: "nib" | "npwp", on: boolean) => void;
}

/**
 * Formulir koreksi data perusahaan saat Ditolak (CR-036; label selalu terlihat, galat per field terhubung lewat
 * `aria-describedby`, DG §7). NIB/NPWP tersimpan hanya tampil tersamar (Q23 = A); kotak "Ubah NIB"/"Ubah NPWP"
 * membuka isian kosong untuk nilai lengkap yang baru. Tanpa `maxLength` di NIB/NPWP (tempelan berformat tidak
 * terpotong diam-diam, sama dengan formulir daftar).
 */
function FormKoreksi({ ref, profil, isian, ubah, galat, onIsian, onUbah }: FormKoreksiProps) {
  const teks = (f: "legal_name" | "office_address" | "office_phone", tipe: "text" | "tel", autoComplete: string) => {
    const g = galat[f];
    return (
      <TextField
        name={`koreksi-${f}`}
        label={LABEL_KOREKSI[f]}
        type={tipe}
        value={isian[f]}
        maxLength={MAKS_KOREKSI[f]}
        autoComplete={autoComplete}
        onChange={(e) => {
          onIsian(f, e.currentTarget.value);
        }}
        {...(g === undefined ? {} : { error: g })}
      />
    );
  };
  const tersamar = (f: "nib" | "npwp", labelSaatIni: string) => {
    const g = galat[f];
    const petunjuk = PETUNJUK_KOREKSI[f];
    return (
      <div className="kp-fieldset">
        <p className="kp-kecil">
          {labelSaatIni}: {termasking(profil[f])}
        </p>
        <Checkbox
          name={`ubah-${f}`}
          label={f === "nib" ? "Ubah NIB" : "Ubah NPWP"}
          checked={ubah[f]}
          onChange={(e) => {
            onUbah(f, e.currentTarget.checked);
          }}
        />
        {ubah[f] ? (
          <TextField
            name={`koreksi-${f}`}
            label={LABEL_KOREKSI[f]}
            value={isian[f]}
            inputMode="numeric"
            autoComplete="off"
            required
            onChange={(e) => {
              onIsian(f, e.currentTarget.value);
            }}
            {...(petunjuk === undefined ? {} : { hint: petunjuk })}
            {...(g === undefined ? {} : { error: g })}
          />
        ) : null}
      </div>
    );
  };
  return (
    <fieldset ref={ref} className="kp-fieldset">
      <legend>Koreksi data perusahaan</legend>
      <p className="kp-kecil">
        Ubah hanya data yang salah. Koreksi dikirim bersama pengajuan ulang dan tercatat di riwayat audit.
      </p>
      {teks("legal_name", "text", "organization")}
      {tersamar("nib", "NIB saat ini")}
      {tersamar("npwp", "NPWP saat ini")}
      {teks("office_address", "text", "street-address")}
      {teks("office_phone", "tel", "tel")}
    </fieldset>
  );
}

interface BarisDokumenProps {
  readonly jenis: JenisDokumen;
  readonly status: StatusDokumen;
  readonly api: KlienVerifikasi;
  readonly onTerima: (r: Ringkasan) => void;
  readonly onGalatAkses: (g: Gagal) => void;
}

function BarisDokumen({ jenis, status, api, onTerima, onGalatAkses }: BarisDokumenProps) {
  const id = useId();
  const label = labelDokumen(jenis);
  const bantuan = DOKUMEN.find((d) => d.jenis === jenis)?.bantuan ?? "";
  const isian = useRef<HTMLInputElement>(null);
  const [berkas, setBerkas] = useState<File | null>(null);
  const [galat, setGalat] = useState<string | null>(null);
  const [sibuk, setSibuk] = useState(false);
  const [unduhSibuk, setUnduhSibuk] = useState(false);
  const [unduh, setUnduh] = useState<UrlUnduh | null>(null);
  const [galatUnduh, setGalatUnduh] = useState<string | null>(null);

  const tampilGalat = (pesan: string) => {
    setGalat(pesan);
    // Fokus ke isian galat (DG §5.3, A12); pesan terhubung lewat aria-describedby.
    isian.current?.focus();
  };

  const pilih = (e: ChangeEvent<HTMLInputElement>) => {
    setGalat(null);
    setBerkas(e.target.files?.[0] ?? null);
  };

  const unggah = async () => {
    const salah = periksaBerkas(berkas);
    if (salah !== null || berkas === null) {
      tampilGalat(salah ?? "Pilih berkas terlebih dahulu.");
      return;
    }
    setSibuk(true);
    const h = await api.unggah(jenis, berkas);
    setSibuk(false);
    if (h.ok) {
      setBerkas(null);
      setUnduh(null);
      if (isian.current !== null) isian.current.value = "";
      onTerima(h.data);
    } else if (adalahGalatAkses(h)) {
      onGalatAkses(h);
    } else {
      tampilGalat(h.pesan);
    }
  };

  const mintaUnduh = async () => {
    setGalatUnduh(null);
    setUnduhSibuk(true);
    const h = await api.urlUnduh(jenis);
    setUnduhSibuk(false);
    // Skema selain https (http hanya development) tidak pernah dirender sebagai tautan (mis. `javascript:`).
    if (h.ok && urlUnduhAman(h.data.url)) setUnduh(h.data);
    else if (h.ok) setGalatUnduh("Tautan dokumen tidak dapat dibuka. Coba lagi beberapa saat lagi.");
    else if (adalahGalatAkses(h)) onGalatAkses(h);
    else setGalatUnduh(h.status === 404 ? "Dokumen belum diunggah." : h.pesan);
  };

  const galatId = `${id}-galat`;
  const terjelaskan = [`${id}-bantuan`, "petunjuk-berkas", galat === null ? null : galatId].filter(Boolean).join(" ");

  return (
    <div className="kp-dokumen" role="group" aria-labelledby={`${id}-judul`}>
      <div className="kp-baris-status">
        <h3 id={`${id}-judul`}>{label}</h3>
        <Badge tone={status.uploaded ? "success" : "neutral"}>{status.uploaded ? "Terunggah" : "Belum diunggah"}</Badge>
        {status.uploaded && status.uploaded_at !== null && status.uploaded_at !== undefined ? (
          <span className="kp-kecil">Diunggah {formatWib(status.uploaded_at)}</span>
        ) : null}
      </div>
      <p id={`${id}-bantuan`} className="kp-kecil">
        {bantuan}
      </p>
      <div className="rs-field">
        <label className="rs-field__label" htmlFor={`${id}-berkas`}>
          {status.uploaded ? `Berkas pengganti ${label}` : `Berkas ${label}`}
        </label>
        <input
          ref={isian}
          id={`${id}-berkas`}
          type="file"
          accept={ACCEPT}
          className="kp-berkas"
          aria-invalid={galat === null ? undefined : true}
          aria-describedby={terjelaskan}
          onChange={pilih}
        />
        {berkas === null ? null : (
          <p className="rs-field__hint">
            Dipilih: {berkas.name} ({formatUkuran(berkas.size)})
          </p>
        )}
        {galat === null ? null : (
          <p id={galatId} className="rs-field__error">
            <span aria-hidden="true" className="rs-field__error-icon">
              !
            </span>
            <span>{galat}</span>
          </p>
        )}
      </div>
      <div className="kp-baris-aksi kp-kiri">
        <Button variant="secondary" loading={sibuk} loadingLabel="Mengunggah..." onClick={() => void unggah()}>
          {status.uploaded ? "Ganti dokumen" : "Unggah"}
        </Button>
        {status.uploaded ? (
          <Button variant="text" loading={unduhSibuk} loadingLabel="Menyiapkan..." onClick={() => void mintaUnduh()}>
            {`Unduh ${label}`}
          </Button>
        ) : null}
      </div>
      {unduh === null ? null : (
        <p className="kp-kecil" role="status">
          {/* URL bertanda tangan berumur pendek (ADR-018): tanpa referrer, tidak disimpan di riwayat aplikasi. */}
          <a href={unduh.url} target="_blank" rel="noopener noreferrer">
            Buka {label}
          </a>{" "}
          (tautan berlaku sampai {formatWib(unduh.expires_at)})
        </p>
      )}
      {galatUnduh === null ? null : (
        <p className="rs-field__error" role="alert">
          <span aria-hidden="true" className="rs-field__error-icon">
            !
          </span>
          <span>{galatUnduh}</span>
        </p>
      )}
    </div>
  );
}
