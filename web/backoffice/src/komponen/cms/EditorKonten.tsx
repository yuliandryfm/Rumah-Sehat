"use client";

import { Alert, Button, Card, TextField } from "@rs/ui";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useId, useMemo, useState, type ReactNode } from "react";
import { buatKlienCms, type Gagal, type Hasil, type KlienCms } from "../../lib/cms/klien.ts";
import {
  LABEL_HALAMAN,
  LABEL_JENIS,
  LABEL_KANAL,
  LABEL_STATUS,
  URUTAN_HALAMAN,
  URUTAN_JENIS,
  URUTAN_KANAL,
  bolehDiarsipkan,
  bolehDihapus,
  bolehDijadwalkan,
  bolehDiubah,
  type HalamanSitus,
  type JenisKonten,
  type Kanal,
  type Konten,
} from "../../lib/cms/model.ts";
import { DialogAlasan } from "./DialogAlasan.tsx";
import { LayarTanpaAkses } from "./LayarTanpaAkses.tsx";
import { PratinjauAman } from "./PratinjauAman.tsx";

export interface EditorKontenProps {
  readonly csrfToken: string;
  /** Tanpa `kontenId`: buat konten baru. */
  readonly kontenId?: string;
  /** Hanya untuk test. */
  readonly klien?: KlienCms;
}

/** Zona waktu tetap WIB (UTC+7); API meminta ISO 8601 dengan offset. */
const OFFSET_WIB = "+07:00";
const KE_ISO = (lokal: string): string | null => (lokal === "" ? null : `${lokal}:00${OFFSET_WIB}`);

function Pilih<T extends string>({
  label,
  nilai,
  opsi,
  disabled,
  onUbah,
}: {
  label: string;
  nilai: T;
  opsi: readonly { v: T; l: string }[];
  disabled: boolean;
  onUbah: (v: T) => void;
}) {
  const id = useId();
  return (
    <div className="rs-field">
      <label className="rs-field__label" htmlFor={id}>
        {label}
      </label>
      <select id={id} className="rs-field__input" value={nilai} disabled={disabled} onChange={(e) => { onUbah(e.target.value as T); }}>
        {opsi.map((o) => (
          <option key={o.v} value={o.v}>
            {o.l}
          </option>
        ))}
      </select>
    </div>
  );
}

function GalatTeks({ children }: { children: ReactNode }) {
  return (
    <p className="rs-field__error">
      <span aria-hidden="true" className="rs-field__error-icon">
        !
      </span>
      <span>{children}</span>
    </p>
  );
}

type Dialog = null | "hapus" | "tayang";

/**
 * Editor konten (D-C07 butir 2, wireframe C14 layar 2): judul, jenis, kanal, halaman situs, isi Markdown,
 * pratinjau terisolasi, gambar dengan teks alternatif wajib, jadwal, arsip, hapus. Hanya draf yang dapat
 * diubah; konten lain tampil baca-saja (API menjawab 409 untuk ubah).
 */
export function EditorKonten({ csrfToken, kontenId, klien }: EditorKontenProps) {
  const api = useMemo(() => klien ?? buatKlienCms(csrfToken), [klien, csrfToken]);
  const router = useRouter();
  const baru = kontenId === undefined;
  const [konten, setKonten] = useState<Konten | null>(null);
  const [memuat, setMemuat] = useState(!baru);
  const [judul, setJudul] = useState("");
  const [jenis, setJenis] = useState<JenisKonten>("info_page");
  const [kanal, setKanal] = useState<Kanal>("site");
  const [halaman, setHalaman] = useState<HalamanSitus>("about");
  const [isi, setIsi] = useState("");
  const [teksAlt, setTeksAlt] = useState("");
  const [mulai, setMulai] = useState("");
  const [selesai, setSelesai] = useState("");
  const [galat, setGalat] = useState<Gagal | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [sibuk, setSibuk] = useState(false);
  const [dialog, setDialog] = useState<Dialog>(null);
  const [berkas, setBerkas] = useState<File | null>(null);
  const [galatAlt, setGalatAlt] = useState<string | null>(null);
  const idBerkas = useId();
  const idIsi = useId();
  const idMulai = useId();
  const idSelesai = useId();

  const terapkan = (k: Konten) => {
    setKonten(k);
    setJudul(k.title);
    setJenis(k.content_type);
    setKanal(k.channel);
    if (k.site_page !== null && k.site_page !== undefined) setHalaman(k.site_page);
    setIsi(k.body ?? "");
    setTeksAlt(k.image_alt_text ?? "");
  };

  useEffect(() => {
    if (kontenId === undefined) return;
    let batal = false;
    void api.ambil(kontenId).then((h) => {
      if (batal) return;
      setMemuat(false);
      if (h.ok) terapkan(h.data);
      else setGalat(h);
    });
    return () => {
      batal = true;
    };
  }, [api, kontenId]);

  if (galat?.status === 403) return <LayarTanpaAkses />;
  if (galat?.status === 404 && konten === null) {
    return (
      <>
        <h1>Konten tidak ditemukan</h1>
        <p>Konten mungkin sudah dihapus. Kembali ke <Link href="/konten">daftar konten</Link>.</p>
      </>
    );
  }
  if (memuat) return <Card loading aria-label="Memuat konten" />;

  const terkunci = konten !== null && !bolehDiubah(konten);
  const untukSitus = kanal !== "app";
  const galatJudul = galat?.isian.filter((e) => e.field === "title") ?? [];
  const galatIsi = galat?.isian.filter((e) => e.field === "body") ?? [];
  const galatLain = galat?.isian.filter((e) => e.field !== "title" && e.field !== "body") ?? [];

  const permintaan = () => ({
    title: judul.trim(),
    content_type: jenis,
    channel: kanal,
    site_page: untukSitus ? halaman : null,
    body: isi,
    category: konten?.category ?? null,
    link_target_type: konten?.link_target_type ?? null,
    link_target_ref: konten?.link_target_ref ?? null,
    image_alt_text: teksAlt.trim() === "" ? null : teksAlt.trim(),
  });

  const kerjakan = async <T,>(p: Promise<Hasil<T>>, berhasil: (d: T) => void) => {
    setSibuk(true);
    setInfo(null);
    const h = await p;
    setSibuk(false);
    if (!h.ok) {
      setGalat(h);
      setDialog(null);
      return false;
    }
    setGalat(null);
    berhasil(h.data);
    return true;
  };

  const simpan = async () => {
    if (judul.trim() === "") {
      setGalat({ ok: false, status: 422, pesan: "Periksa isian yang ditandai.", kode: null, isian: [{ field: "title", code: "required", message: "Judul wajib diisi (1-200 karakter)." }] });
      return;
    }
    if (baru) {
      await kerjakan(api.buat(permintaan()), (k) => { router.replace(`/konten/${k.content_id}`); });
    } else {
      await kerjakan(api.ubah(kontenId, permintaan()), (k) => {
        terapkan(k);
        setInfo("Draf disimpan.");
      });
    }
  };

  const tayangkan = async () => {
    if (kontenId === undefined) return;
    setDialog(null);
    // Simpan isian terbaru dulu supaya yang tayang sama dengan yang terlihat di pratinjau.
    const tersimpan = await kerjakan(api.ubah(kontenId, permintaan()), terapkan);
    if (!tersimpan) return;
    await kerjakan(api.jadwalkan(kontenId, KE_ISO(mulai), KE_ISO(selesai)), (k) => {
      terapkan(k);
      setInfo(k.display_status === "scheduled" ? "Konten dijadwalkan." : "Konten ditayangkan.");
    });
  };

  const arsipkan = () => {
    if (kontenId === undefined) return Promise.resolve();
    return kerjakan(api.arsipkan(kontenId), (k) => {
      terapkan(k);
      setInfo("Konten diarsipkan dan tidak tampil lagi.");
    }).then(() => undefined);
  };

  const hapus = async (alasan: string) => {
    if (kontenId === undefined) return;
    const ok = await kerjakan(api.hapus(kontenId, alasan), () => undefined);
    if (ok) router.replace("/konten");
  };

  const unggah = async () => {
    if (kontenId === undefined || berkas === null) return;
    if (teksAlt.trim() === "") {
      setGalatAlt("Teks alternatif wajib diisi sebelum gambar dipasang. Jelaskan isi gambar secara singkat.");
      return;
    }
    setGalatAlt(null);
    await kerjakan(api.unggahGambar(kontenId, berkas, teksAlt.trim()), (k) => {
      terapkan(k);
      setBerkas(null);
      setInfo("Gambar dipasang.");
    });
  };

  return (
    <>
      <div className="bo-judul-baris">
        <h1>{baru ? "Buat konten" : `Ubah konten: ${konten?.title ?? ""}`}</h1>
        {konten === null ? null : <span className="bo-kecil">{LABEL_STATUS[konten.display_status]}</span>}
      </div>
      {terkunci ? (
        <Alert variant="info" title="Konten tidak dapat diubah">
          Hanya draf yang dapat diubah. Untuk konten berstatus {LABEL_STATUS[konten.display_status]}, buat konten baru sebagai versi berikutnya.
        </Alert>
      ) : null}
      {info === null ? null : <Alert variant="success" title={info} />}
      {galat === null ? null : (
        <Alert variant="danger" title="Tindakan gagal">
          {galat.pesan}
        </Alert>
      )}
      {dialog === "hapus" ? (
        <DialogAlasan
          judul={`Hapus konten "${konten?.title ?? ""}"?`}
          penjelasan="Konten tidak tampil lagi di kanal terpilih dan dapat dipulihkan dari filter Terhapus pada daftar. Tindakan ini dicatat."
          labelAksi="Hapus"
          sibuk={sibuk}
          onBatal={() => { setDialog(null); }}
          onKirim={(a) => void hapus(a)}
        />
      ) : null}
      {dialog === "tayang" ? (
        <div role="alertdialog" aria-modal="true" aria-labelledby={`${idIsi}-t`} className="bo-dialog">
          <h2 id={`${idIsi}-t`}>{mulai === "" ? "Tayangkan konten ini?" : "Jadwalkan konten ini?"}</h2>
          <p>
            {untukSitus
              ? `Bila halaman "${LABEL_HALAMAN[halaman]}" sedang memiliki versi tayang, versi itu berakhir dan digantikan versi ini.`
              : `Konten tampil di kanal ${LABEL_KANAL[kanal]}.`}
          </p>
          <div className="bo-aksi">
            <Button variant="secondary" onClick={() => { setDialog(null); }}>
              Batal
            </Button>
            <Button loading={sibuk} onClick={() => void tayangkan()}>
              {mulai === "" ? "Tayangkan" : "Jadwalkan"}
            </Button>
          </div>
        </div>
      ) : null}
      <div className="bo-editor">
        <form
          className="bo-form"
          aria-label="Isian konten"
          onSubmit={(e) => {
            e.preventDefault();
            void simpan();
          }}
        >
          <TextField
            label="Judul"
            value={judul}
            maxLength={200}
            disabled={terkunci}
            onChange={(e) => { setJudul(e.target.value); }}
            {...(galatJudul[0] === undefined ? {} : { error: galatJudul[0].message })}
          />
          <Pilih label="Jenis" nilai={jenis} disabled={terkunci} opsi={URUTAN_JENIS.map((v) => ({ v, l: LABEL_JENIS[v] }))} onUbah={setJenis} />
          <Pilih label="Kanal" nilai={kanal} disabled={terkunci} opsi={URUTAN_KANAL.map((v) => ({ v, l: LABEL_KANAL[v] }))} onUbah={setKanal} />
          {untukSitus ? (
            <Pilih label="Halaman situs tujuan" nilai={halaman} disabled={terkunci} opsi={URUTAN_HALAMAN.map((v) => ({ v, l: LABEL_HALAMAN[v] }))} onUbah={setHalaman} />
          ) : null}
          <div className="rs-field">
            <label className="rs-field__label" htmlFor={idIsi}>
              Isi (Markdown)
            </label>
            <textarea
              id={idIsi}
              className="rs-field__input bo-textarea bo-isi"
              rows={12}
              value={isi}
              disabled={terkunci}
              aria-invalid={galatIsi.length > 0 ? true : undefined}
              aria-describedby={`${idIsi}-h`}
              onChange={(e) => { setIsi(e.target.value); }}
            />
            <p id={`${idIsi}-h`} className="rs-field__hint">
              Hanya Markdown. HTML mentah, skrip, dan sematan tidak didukung.
            </p>
            {/* Pesan dari API dirender sebagai teks (React meng-escape), tidak pernah sebagai HTML. */}
            {galatIsi.map((e, i) => (
              <GalatTeks key={i}>{e.message}</GalatTeks>
            ))}
          </div>
          {galatLain.length === 0 ? null : (
            <ul className="bo-galat-daftar" aria-label="Galat isian lain">
              {galatLain.map((e, i) => (
                <li key={i}>
                  <GalatTeks>{e.message}</GalatTeks>
                </li>
              ))}
            </ul>
          )}

          {baru ? (
            <p className="bo-bantu">Simpan draf terlebih dahulu untuk memasang gambar dan menjadwalkan tayang.</p>
          ) : (
            <>
              <fieldset className="bo-fieldset" disabled={terkunci}>
                <legend>Gambar (opsional)</legend>
                {konten?.image_file_id === null || konten?.image_file_id === undefined ? null : (
                  <p className="bo-kecil">Gambar terpasang. ID berkas: {konten.image_file_id}</p>
                )}
                <div className="rs-field">
                  <label className="rs-field__label" htmlFor={idBerkas}>
                    Berkas gambar (JPG atau PNG, maksimal 5 MiB)
                  </label>
                  <input
                    id={idBerkas}
                    type="file"
                    accept="image/jpeg,image/png"
                    className="rs-field__input"
                    onChange={(e) => { setBerkas(e.target.files?.[0] ?? null); }}
                  />
                </div>
                <TextField
                  label="Teks alternatif gambar"
                  hint="Wajib sebelum konten dapat ditayangkan."
                  value={teksAlt}
                  maxLength={300}
                  onChange={(e) => { setTeksAlt(e.target.value); }}
                  {...(galatAlt === null ? {} : { error: galatAlt })}
                />
                <Button variant="secondary" disabled={berkas === null} loading={sibuk && berkas !== null} onClick={() => void unggah()}>
                  Unggah dan pasang gambar
                </Button>
                {berkas === null ? <p className="rs-field__hint">Pilih berkas gambar lebih dulu.</p> : null}
              </fieldset>
              <fieldset className="bo-fieldset" disabled={terkunci || (konten !== null && !bolehDijadwalkan(konten))}>
                <legend>Jadwal tayang (opsional, WIB)</legend>
                <div className="rs-field">
                  <label className="rs-field__label" htmlFor={idMulai}>
                    Mulai tayang
                  </label>
                  <input id={idMulai} type="datetime-local" className="rs-field__input" value={mulai} onChange={(e) => { setMulai(e.target.value); }} />
                  <p className="rs-field__hint">Kosongkan untuk tayang sekarang.</p>
                </div>
                <div className="rs-field">
                  <label className="rs-field__label" htmlFor={idSelesai}>
                    Selesai tayang
                  </label>
                  <input id={idSelesai} type="datetime-local" className="rs-field__input" value={selesai} onChange={(e) => { setSelesai(e.target.value); }} />
                </div>
              </fieldset>
            </>
          )}
          <p className="bo-bantu">Promo hanya informasi: tidak mengubah harga di aplikasi maupun katalog korporat.</p>
          <div className="bo-aksi">
            {terkunci ? null : (
              <Button type="submit" variant="secondary" loading={sibuk && dialog === null && berkas === null}>
                Simpan draf
              </Button>
            )}
            {baru || konten === null || !bolehDijadwalkan(konten) ? null : (
              <Button onClick={() => { setDialog("tayang"); }}>{mulai === "" ? "Tayangkan" : "Jadwalkan"}</Button>
            )}
            {konten !== null && bolehDiarsipkan(konten) ? (
              <Button variant="secondary" disabled={sibuk} onClick={() => void arsipkan()}>
                Arsipkan
              </Button>
            ) : null}
            {konten !== null && bolehDihapus(konten) ? (
              <Button variant="destructive" onClick={() => { setDialog("hapus"); }}>
                Hapus
              </Button>
            ) : null}
            <Link href="/konten" className="rs-btn rs-btn--text">
              Kembali ke daftar
            </Link>
          </div>
        </form>
        <section className="bo-pratinjau-bungkus" aria-labelledby={`${idIsi}-p`}>
          <h2 id={`${idIsi}-p`}>Pratinjau</h2>
          <p className="bo-kecil">Terisolasi: skrip apa pun di pratinjau tidak dapat memakai sesi backoffice.</p>
          <PratinjauAman judul={judul} markdown={isi} />
        </section>
      </div>
    </>
  );
}
