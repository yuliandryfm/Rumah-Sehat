"use client";

import { Alert, Badge, Button, Skeleton, TextField } from "@rs/ui";
import { useCallback, useEffect, useMemo, useRef, useState, type SyntheticEvent } from "react";
import {
  KODE_SUDAH_NONAKTIF,
  buatKlienHr,
  type AnggotaHr,
  type Gagal,
  type KlienHr,
} from "../../lib/hr/klien.ts";
import {
  EMAIL_MAKS,
  LABEL_STATUS,
  PESAN_UNDANGAN_DIPROSES,
  dapatDinonaktifkan,
  emailTampakValid,
  namaTampil,
} from "../../lib/hr/model.ts";
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
 * saat dibatalkan. "Undang HR" (AC2) membuka formulir email sebaris; jawaban 202 server seragam untuk email baru
 * dan email terpakai, dan UI menampilkan satu pesan tetap tanpa menyebut email (anti-enumerasi, KEPUTUSAN-306).
 */
export function KelolaHr({ csrfToken, klien, arahkan }: KelolaHrProps) {
  const api = useMemo(() => klien ?? buatKlienHr(csrfToken), [klien, csrfToken]);
  const [anggota, setAnggota] = useState<readonly AnggotaHr[] | null>(null);
  const [galat, setGalat] = useState<Gagal | null>(null);
  const [galatAksi, setGalatAksi] = useState<Gagal | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [target, setTarget] = useState<AnggotaHr | null>(null);
  const [sibuk, setSibuk] = useState(false);
  const [formUndang, setFormUndang] = useState(false);
  const [emailUndang, setEmailUndang] = useState("");
  const [galatEmail, setGalatEmail] = useState<string | null>(null);
  const [galatUndang, setGalatUndang] = useState<Gagal | null>(null);
  const [undangSibuk, setUndangSibuk] = useState(false);
  const [infoUndang, setInfoUndang] = useState(false);
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
  if (adalahGalatAkses(galatUndang)) return <GalatAkses galat={galatUndang} {...(arahkan === undefined ? {} : { arahkan })} />;
  if (adalahGalatAkses(galatAksi)) return <GalatAkses galat={galatAksi} {...(arahkan === undefined ? {} : { arahkan })} />;

  const bukaUndang = () => {
    setInfoUndang(false);
    setGalatUndang(null);
    setGalatEmail(null);
    setFormUndang(true);
  };
  const tutupUndang = () => {
    setFormUndang(false);
    setEmailUndang("");
    setGalatEmail(null);
    setGalatUndang(null);
  };
  const kirimUndang = async (e: SyntheticEvent) => {
    e.preventDefault();
    if (undangSibuk) return;
    setGalatUndang(null);
    setInfoUndang(false);
    if (!emailTampakValid(emailUndang)) {
      setGalatEmail("Masukkan email yang valid, misalnya nama@perusahaan.example.");
      return;
    }
    setGalatEmail(null);
    setUndangSibuk(true);
    const h = await api.undang(emailUndang.trim());
    setUndangSibuk(false);
    if (!h.ok) {
      if (h.status === 422) setGalatEmail(h.pesan);
      else setGalatUndang(h);
      return;
    }
    // Pesan tetap, tidak bergantung isi respons: sama untuk email baru dan email yang sudah terdaftar.
    setFormUndang(false);
    setEmailUndang("");
    setInfoUndang(true);
    setInfo(null);
    await muat();
  };
  const mulaiNonaktif = (a: AnggotaHr, el: HTMLElement) => {
    pemicu.current = el;
    setInfo(null);
    setInfoUndang(false);
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
        <h1>Tim HR</h1>
        <Button variant="primary" onClick={bukaUndang} disabled={formUndang} aria-expanded={formUndang}>
          Undang HR
        </Button>
      </div>

      {formUndang ? (
        <form className="kp-konfirmasi" aria-labelledby="judul-undang" noValidate onSubmit={(e) => void kirimUndang(e)}>
          <h2 id="judul-undang">Undang HR</h2>
          <TextField
            label="Email HR yang diundang"
            type="email"
            name="email"
            autoComplete="off"
            autoFocus
            maxLength={EMAIL_MAKS}
            required
            value={emailUndang}
            hint="Penerima mendapat tautan aktivasi (password dan TOTP) yang berlaku 24 jam dan sekali pakai."
            {...(galatEmail === null ? {} : { error: galatEmail })}
            onChange={(e) => {
              setEmailUndang(e.target.value);
              setGalatEmail(null);
            }}
          />
          {galatUndang === null ? null : (
            <Alert variant="danger" title="Undangan tidak dapat dikirim">
              {galatUndang.pesan}
            </Alert>
          )}
          <div className="kp-baris-aksi">
            <Button variant="secondary" type="button" onClick={tutupUndang} disabled={undangSibuk}>
              Batal
            </Button>
            <Button variant="primary" type="submit" loading={undangSibuk} loadingLabel="Mengirim...">
              Kirim undangan
            </Button>
          </div>
        </form>
      ) : null}

      {infoUndang ? (
        <Alert variant="success" title="Undangan diproses">
          {PESAN_UNDANGAN_DIPROSES}
        </Alert>
      ) : null}
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
