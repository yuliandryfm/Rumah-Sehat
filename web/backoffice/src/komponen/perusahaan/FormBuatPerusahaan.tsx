"use client";

import { Alert, Button, TextField } from "@rs/ui";
import { useState, type SyntheticEvent } from "react";
import { KODE_CSRF_DITOLAK, KODE_EMAIL_HR_DIPAKAI, KODE_NIB_TERDAFTAR, type Gagal, type KlienPerusahaan } from "../../lib/perusahaan/klien.ts";
import { BATAS_FORM, validasiForm, type KolomForm, type PermintaanBuatPerusahaan } from "../../lib/perusahaan/model.ts";

export interface FormBuatPerusahaanProps {
  readonly api: KlienPerusahaan;
  /** 401 atau 403 selain CSRF: pemanggil menampilkan layar tanpa akses, tanpa data. */
  readonly onGalatAkses: (galat: Gagal) => void;
  /** Dipanggil setelah perusahaan dibuat (pemanggil memuat ulang antrean dan menampilkan hasil). */
  readonly onSelesai: () => void;
  readonly onBatal: () => void;
}

const KOSONG: PermintaanBuatPerusahaan = { legal_name: "", nib: "", npwp: "", office_address: "", office_phone: "", hr_email: "" };

const KOLOM: readonly { readonly nama: KolomForm; readonly label: string; readonly type?: string; readonly hint?: string }[] = [
  { nama: "legal_name", label: "Nama badan usaha" },
  { nama: "nib", label: "NIB", hint: "Tanpa spasi. Format resmi NIB belum ditetapkan; server yang memeriksa." },
  { nama: "npwp", label: "NPWP", hint: "Tanpa spasi." },
  { nama: "office_address", label: "Alamat kantor" },
  { nama: "office_phone", label: "Telepon kantor" },
  { nama: "hr_email", label: "Email HR pertama", type: "email", hint: "Penerima tautan aktivasi (password dan TOTP) yang berlaku 24 jam dan sekali pakai." },
];

/**
 * Formulir "Buat perusahaan" Verifikator (wireframe C13, US-1903 AC1/AC4). Perusahaan langsung Aktif dan HR
 * pertama menerima undangan yang diproses beberapa saat kemudian. Lampiran dokumen perusahaan belum ada:
 * menunggu F1-T102, ditandai jujur di layar. Isian (NIB, NPWP, email) tidak pernah dicatat ke console.
 */
export function FormBuatPerusahaan({ api, onGalatAkses, onSelesai, onBatal }: FormBuatPerusahaanProps) {
  const [isian, setIsian] = useState<PermintaanBuatPerusahaan>(KOSONG);
  const [galatKolom, setGalatKolom] = useState<Partial<Record<KolomForm, string>>>({});
  const [galat, setGalat] = useState<Gagal | null>(null);
  const [sibuk, setSibuk] = useState(false);

  const ubah = (k: KolomForm, v: string) => {
    setIsian((l) => ({ ...l, [k]: v }));
    setGalatKolom((l) => ({ ...l, [k]: undefined }));
  };

  const kirim = async (e: SyntheticEvent) => {
    e.preventDefault();
    if (sibuk) return;
    setGalat(null);
    const g = validasiForm(isian);
    if (Object.keys(g).length > 0) {
      setGalatKolom(g);
      return;
    }
    setGalatKolom({});
    setSibuk(true);
    const h = await api.buat(isian);
    setSibuk(false);
    if (h.ok) {
      setIsian(KOSONG);
      onSelesai();
      return;
    }
    if (h.status === 401 || (h.status === 403 && h.kode !== KODE_CSRF_DITOLAK)) {
      onGalatAkses(h);
      return;
    }
    if (h.kode === KODE_NIB_TERDAFTAR) setGalatKolom({ nib: h.pesan });
    else if (h.kode === KODE_EMAIL_HR_DIPAKAI) setGalatKolom({ hr_email: h.pesan });
    else setGalat(h);
  };

  return (
    <form className="bo-form" aria-labelledby="judul-buat-perusahaan" noValidate onSubmit={(e) => void kirim(e)}>
      <h2 id="judul-buat-perusahaan">Buat perusahaan</h2>
      <p className="bo-bantu">
        Perusahaan langsung berstatus Aktif tanpa antrean verifikasi, dan HR pertama menerima undangan aktivasi.
      </p>
      <p className="bo-bantu" data-testid="dokumen-menyusul">
        Unggah dokumen perusahaan (NIB, NPWP, bukti kewenangan) menyusul: menunggu fitur unggah dokumen (F1-T102).
        Perusahaan dapat dibuat tanpa lampiran untuk saat ini.
      </p>
      {KOLOM.map((k) => {
        const pesan = galatKolom[k.nama];
        return (
        <TextField
          key={k.nama}
          label={k.label}
          name={k.nama}
          {...(k.type === undefined ? {} : { type: k.type })}
          {...(k.hint === undefined ? {} : { hint: k.hint })}
          {...(pesan === undefined ? {} : { error: pesan })}
          autoComplete="off"
          required
          maxLength={BATAS_FORM[k.nama]}
          value={isian[k.nama]}
          onChange={(e) => { ubah(k.nama, e.target.value); }}
        />
        );
      })}
      {galat === null ? null : (
        <Alert variant="danger" title="Perusahaan tidak dapat dibuat">
          {galat.pesan}
        </Alert>
      )}
      <div className="bo-aksi">
        <Button variant="primary" type="submit" loading={sibuk} loadingLabel="Menyimpan...">
          Buat perusahaan
        </Button>
        <Button variant="text" type="button" onClick={onBatal} disabled={sibuk}>
          Batal
        </Button>
      </div>
    </form>
  );
}
