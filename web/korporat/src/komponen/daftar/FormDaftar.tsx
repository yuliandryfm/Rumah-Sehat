"use client";

import { Alert, Button, Checkbox, Skeleton, TextField } from "@rs/ui";
import { useCallback, useEffect, useMemo, useRef, useState, type SyntheticEvent } from "react";
import { KODE_TEKS_BERUBAH, buatKlienDaftar, type KlienDaftar, type TeksPersetujuan } from "../../lib/daftar/klien.ts";
import {
  ISIAN_KOSONG,
  ISIAN_LANGKAH,
  LABEL,
  LANGKAH,
  MAKS,
  PETUNJUK,
  langkahDari,
  periksaLangkah,
  type Galat,
  type Isian,
  type NamaIsian,
  type NomorLangkah,
} from "../../lib/daftar/model.ts";

export interface FormDaftarProps {
  /** Hanya untuk test. */
  readonly klien?: KlienDaftar;
}

type Teks = { readonly status: "memuat" } | { readonly status: "ada"; readonly teks: TeksPersetujuan } | { readonly status: "gagal"; readonly pesan: string };

/**
 * Formulir pendaftaran perusahaan mandiri bertahap (F1-T103; US-1901 AC1, AC3-AC5, AC7; brief KR-01-KR-04;
 * wireframe K01 layar 2-5): data perusahaan, data HR, teks persetujuan versi berlaku + ringkasan, kirim, lalu layar
 * "Periksa email Anda" dengan pesan seragam dari server (sama untuk email/NIB baru dan terdaftar, anti-enumerasi).
 * Tanpa sesi dan tanpa cookie: memanggil API publik langsung (Q30 = C, KEPUTUSAN-372). Isian hanya di memori.
 */
export function FormDaftar({ klien }: FormDaftarProps) {
  const api = useMemo(() => klien ?? buatKlienDaftar(), [klien]);
  const [langkah, setLangkah] = useState<NomorLangkah>(0);
  const [isian, setIsian] = useState<Isian>(ISIAN_KOSONG);
  const [galat, setGalat] = useState<Galat>({});
  const [ringkasanGalat, setRingkasanGalat] = useState<string | null>(null);
  const [teks, setTeks] = useState<Teks>({ status: "memuat" });
  const [dicentang, setDicentang] = useState<ReadonlySet<string>>(new Set());
  const [sibuk, setSibuk] = useState(false);
  const [galatKirim, setGalatKirim] = useState<string | null>(null);
  const [selesai, setSelesai] = useState<string | null>(null);
  // Formulir terkunci sampai terhidrasi (review T1 B2): sebelum JS jalan, submit bawaan peramban tidak boleh
  // mengirim isian (NPWP, nama, email) ke mana pun, termasuk ke URL lewat GET.
  const [siap, setSiap] = useState(false);
  const judul = useRef<HTMLHeadingElement>(null);
  const formulir = useRef<HTMLFormElement>(null);
  const pindahFokus = useRef<"judul" | NamaIsian | "consent" | null>(null);

  const muatTeks = useCallback(async () => {
    setTeks({ status: "memuat" });
    const h = await api.teks();
    if (h.ok) setTeks({ status: "ada", teks: h.data });
    else setTeks({ status: "gagal", pesan: h.status === 503 ? "Pendaftaran belum dapat dibuka karena teks persetujuan belum tersedia. Coba lagi nanti." : h.pesan });
  }, [api]);

  useEffect(() => {
    void muatTeks();
  }, [muatTeks]);

  useEffect(() => {
    setSiap(true);
  }, []);

  // Fokus setelah render: ke judul langkah saat berpindah, atau ke isian galat pertama (DG §5.3, A12).
  useEffect(() => {
    const target = pindahFokus.current;
    pindahFokus.current = null;
    if (target === null) return;
    if (target === "judul") judul.current?.focus();
    else if (target === "consent") formulir.current?.querySelector<HTMLInputElement>('input[type="checkbox"]')?.focus();
    else formulir.current?.querySelector<HTMLInputElement>(`[name="${target}"]`)?.focus();
  });

  if (selesai !== null) {
    return (
      <section aria-labelledby="judul-selesai" className="kp-daftar">
        <h1 id="judul-selesai" tabIndex={-1} ref={(el) => el?.focus()}>
          Periksa email Anda
        </h1>
        <Alert variant="info" title="Permintaan pendaftaran diterima">
          {selesai}
        </Alert>
        <p className="kp-kecil">
          Buka tautan di email untuk mengatur password dan autentikator, lalu masuk dan unggah dokumen perusahaan. Bila
          email tidak masuk dalam beberapa menit, periksa folder spam.
        </p>
      </section>
    );
  }

  const ubah = (nama: NamaIsian) => (e: SyntheticEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    const nilai = e.currentTarget.value;
    setIsian((s) => ({ ...s, [nama]: nilai }));
  };

  const tampilkanGalat = (g: Galat, ke: NomorLangkah) => {
    const jumlah = Object.keys(g).length;
    setGalat(g);
    setRingkasanGalat(jumlah === 0 ? null : `Ada ${jumlah.toString()} isian yang perlu diperbaiki.`);
    const urutan: readonly (NamaIsian | "consent")[] = ke === 2 ? ["consent"] : ISIAN_LANGKAH[ke];
    pindahFokus.current = urutan.find((n) => g[n] !== undefined) ?? null;
  };

  const keLangkah = (n: NomorLangkah) => {
    setLangkah(n);
    setRingkasanGalat(null);
    setGalatKirim(null);
    pindahFokus.current = "judul";
  };

  const lanjut = (e: SyntheticEvent) => {
    e.preventDefault();
    if (langkah === 2) {
      void kirim();
      return;
    }
    const g = periksaLangkah(langkah, isian);
    if (Object.keys(g).length > 0) {
      tampilkanGalat(g, langkah);
      return;
    }
    setGalat({});
    keLangkah(langkah === 0 ? 1 : 2);
  };

  const kirim = async () => {
    setGalatKirim(null);
    if (teks.status !== "ada") return;
    const t = teks.teks;
    const belum = t.items.filter((i) => i.is_required && !dicentang.has(i.consent_item_id));
    if (belum.length > 0) {
      tampilkanGalat({ consent: "Centang persetujuan wajib untuk mengirim pendaftaran." }, 2);
      return;
    }
    // Isian langkah 1-2 diperiksa lagi (bisa diubah lewat "Ubah").
    for (const l of [0, 1] as const) {
      const g = periksaLangkah(l, isian);
      if (Object.keys(g).length > 0) {
        setLangkah(l);
        tampilkanGalat(g, l);
        return;
      }
    }
    setSibuk(true);
    const butir = t.items.filter((i) => dicentang.has(i.consent_item_id)).map((i) => i.consent_item_id);
    const h = await api.kirim(isian, t.consent_text_version_id, butir);
    setSibuk(false);
    if (h.ok) {
      setSelesai(h.data.message);
      return;
    }
    if (h.status === 409 && h.kode === KODE_TEKS_BERUBAH) {
      // Versi teks basi: muat ulang teks, kosongkan centang (persetujuan harus diberikan atas teks terbaru).
      setDicentang(new Set());
      setGalatKirim(h.pesan);
      void muatTeks();
      return;
    }
    if (h.status === 422 && Object.keys(h.isian).length > 0) {
      const g: Galat = {};
      for (const [field, pesan] of Object.entries(h.isian)) {
        if (field in LABEL) g[field as NamaIsian] = pesan;
        else if (field.startsWith("consent")) g.consent = pesan;
      }
      const ke = Object.keys(h.isian).map(langkahDari).sort()[0] ?? 2;
      setLangkah(ke);
      tampilkanGalat(g, ke);
      if (Object.keys(g).length === 0) setGalatKirim(h.pesan);
      return;
    }
    setGalatKirim(h.pesan);
  };

  const field = (nama: NamaIsian, tipe: "text" | "email" | "tel" = "text", ekstra: { inputMode?: "numeric"; autoComplete?: string } = {}) => {
    const g = galat[nama];
    const petunjuk = PETUNJUK[nama];
    return (
      <TextField
        key={nama}
        name={nama}
        label={LABEL[nama]}
        type={tipe}
        value={isian[nama]}
        // NIB/NPWP tanpa maxLength: tempelan berformat (titik, tanda hubung) tidak boleh terpotong diam-diam; galat menjelaskan.
        {...(nama === "nib" || nama === "npwp" ? {} : { maxLength: MAKS[nama] })}
        onChange={ubah(nama)}
        required
        {...(petunjuk === undefined ? {} : { hint: petunjuk })}
        {...(g === undefined ? {} : { error: g })}
        {...ekstra}
      />
    );
  };

  return (
    <section aria-labelledby="judul-daftar" className="kp-daftar">
      <h1 id="judul-daftar">Daftarkan perusahaan</h1>
      <ol className="kp-langkah" aria-label="Langkah pendaftaran">
        {LANGKAH.map((l, i) => (
          <li key={l} aria-current={i === langkah ? "step" : undefined} data-selesai={i < langkah ? "true" : undefined}>
            <span aria-hidden="true">{i + 1}</span> {l}
          </li>
        ))}
      </ol>
      <noscript>
        <p>Formulir pendaftaran membutuhkan JavaScript. Aktifkan JavaScript di peramban Anda, lalu muat ulang halaman ini.</p>
      </noscript>
      {/* Lapis kedua bila terkirim tanpa JS: POST ke halaman ini (isian di badan, bukan di URL); fieldset terkunci. */}
      <form ref={formulir} method="post" action="/daftar" noValidate onSubmit={lanjut} className="kp-bagian">
        <fieldset disabled={!siap} className="kp-fieldset">
        <h2 id="judul-langkah" tabIndex={-1} ref={judul}>
          {`Langkah ${(langkah + 1).toString()} dari 3: ${LANGKAH[langkah]}`}
        </h2>
        {ringkasanGalat === null ? null : (
          <p role="alert" className="rs-field__error">
            <span aria-hidden="true" className="rs-field__error-icon">
              !
            </span>
            <span>{ringkasanGalat}</span>
          </p>
        )}
        {langkah === 0 ? (
          <>
            {field("legal_name", "text", { autoComplete: "organization" })}
            {field("nib", "text", { inputMode: "numeric", autoComplete: "off" })}
            {field("npwp", "text", { inputMode: "numeric", autoComplete: "off" })}
            {field("office_address", "text", { autoComplete: "street-address" })}
            {field("office_phone", "tel", { autoComplete: "tel" })}
          </>
        ) : null}
        {langkah === 1 ? (
          <>
            {field("hr_full_name", "text", { autoComplete: "name" })}
            {field("hr_job_title", "text", { autoComplete: "organization-title" })}
            {field("hr_email", "email", { autoComplete: "email" })}
            <p className="kp-kecil">Password tidak diminta di sini. Anda mengatur password dan autentikator lewat tautan aktivasi di email.</p>
          </>
        ) : null}
        {langkah === 2 ? (
          <LangkahPersetujuan
            isian={isian}
            teks={teks}
            dicentang={dicentang}
            galat={galat.consent}
            onCentang={(id, on) => {
              setDicentang((s) => {
                const baru = new Set(s);
                if (on) baru.add(id);
                else baru.delete(id);
                return baru;
              });
            }}
            onUbah={keLangkah}
            onMuatUlang={() => void muatTeks()}
          />
        ) : null}
        {galatKirim === null ? null : (
          <Alert variant="danger" title="Pendaftaran belum terkirim">
            {galatKirim}
          </Alert>
        )}
        <div className="kp-baris-aksi kp-kiri">
          {langkah > 0 ? (
            <Button variant="secondary" onClick={() => { keLangkah(langkah === 2 ? 1 : 0); }}>
              Kembali
            </Button>
          ) : null}
          <Button type="submit" loading={sibuk} loadingLabel="Mengirim...">
            {langkah === 2 ? (galatKirim === null ? "Kirim pendaftaran" : "Coba lagi") : "Lanjut"}
          </Button>
        </div>
        </fieldset>
        {langkah === 0 ? (
          <p className="kp-kecil">
            Sudah punya akun? <a href="/auth/login">Masuk</a>
          </p>
        ) : null}
      </form>
    </section>
  );
}

interface LangkahPersetujuanProps {
  readonly isian: Isian;
  readonly teks: Teks;
  readonly dicentang: ReadonlySet<string>;
  readonly galat: string | undefined;
  readonly onCentang: (id: string, on: boolean) => void;
  readonly onUbah: (n: NomorLangkah) => void;
  readonly onMuatUlang: () => void;
}

function LangkahPersetujuan({ isian, teks, dicentang, galat, onCentang, onUbah, onMuatUlang }: LangkahPersetujuanProps) {
  return (
    <>
      <section aria-labelledby="judul-ringkasan" className="kp-bagian">
        <h3 id="judul-ringkasan">Ringkasan isian</h3>
        <dl className="kp-profil">
          {(["legal_name", "nib", "npwp", "office_address", "office_phone"] as const).map((n) => (
            <Baris key={n} label={LABEL[n]} nilai={isian[n]} />
          ))}
        </dl>
        <div>
          <Button variant="text" onClick={() => { onUbah(0); }}>
            Ubah data perusahaan
          </Button>
        </div>
        <dl className="kp-profil">
          {(["hr_full_name", "hr_job_title", "hr_email"] as const).map((n) => (
            <Baris key={n} label={LABEL[n]} nilai={isian[n]} />
          ))}
        </dl>
        <div>
          <Button variant="text" onClick={() => { onUbah(1); }}>
            Ubah data Anda
          </Button>
        </div>
      </section>
      <section aria-labelledby="judul-persetujuan" className="kp-bagian">
        <h3 id="judul-persetujuan">Persetujuan pemrosesan data</h3>
        {teks.status === "memuat" ? <Skeleton lines={4} /> : null}
        {teks.status === "gagal" ? (
          <Alert variant="danger" title="Teks persetujuan tidak dapat dimuat">
            <p>{teks.pesan}</p>
            <Button variant="secondary" onClick={onMuatUlang}>
              Muat ulang teks
            </Button>
          </Alert>
        ) : null}
        {teks.status === "ada" ? (
          <>
            {teks.teks.draft_notice === null || teks.teks.draft_notice === undefined ? null : (
              <Alert variant="warning" title="Teks draf">
                {teks.teks.draft_notice}
              </Alert>
            )}
            <p className="kp-kecil">Versi teks: {teks.teks.version_label}</p>
            <div className="kp-teks-persetujuan" role="region" aria-label="Teks persetujuan" tabIndex={0}>
              {teks.teks.sections.map((s) => (
                <div key={s.code}>
                  <h4>{s.title}</h4>
                  {/* Teks biasa dari server (bukan HTML): dirender sebagai teks, tidak pernah sebagai markup. */}
                  <p>{s.body}</p>
                </div>
              ))}
              <div>
                <h4>{teks.teks.contact.title}</h4>
                <p>{teks.teks.contact.body}</p>
              </div>
            </div>
            {[...teks.teks.items]
              .sort((a, b) => a.sort_order - b.sort_order)
              .map((b, i) => (
                <Checkbox
                  key={b.consent_item_id}
                  name={`consent-${b.item_code}`}
                  label={b.is_required ? `${b.label} (wajib)` : b.label}
                  checked={dicentang.has(b.consent_item_id)}
                  onChange={(e) => { onCentang(b.consent_item_id, e.currentTarget.checked); }}
                  {...(galat !== undefined && i === 0 ? { error: galat } : {})}
                />
              ))}
          </>
        ) : null}
      </section>
    </>
  );
}

function Baris({ label, nilai }: { label: string; nilai: string }) {
  return (
    <>
      <dt>{label}</dt>
      <dd>{nilai.trim()}</dd>
    </>
  );
}
