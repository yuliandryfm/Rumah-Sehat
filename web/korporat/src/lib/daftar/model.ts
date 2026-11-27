/**
 * Model formulir pendaftaran perusahaan mandiri (F1-T103; US-1901 AC1, AC3; brief KR-01-KR-04; wireframe K01
 * layar 2-5). Validasi di sini hanya bantuan: server (`POST /v1/public/corporate-registrations`) tetap sumber
 * kebenaran. Isian hanya di memori halaman: tidak disimpan di perangkat (DG D-K01, catatan QA N-8). Tanpa
 * password (KEPUTUSAN-303) dan tanpa dokumen (diunggah setelah aktivasi, Q8 A).
 */

export interface Isian {
  legal_name: string;
  nib: string;
  npwp: string;
  office_address: string;
  office_phone: string;
  hr_full_name: string;
  hr_job_title: string;
  hr_email: string;
}

export type NamaIsian = keyof Isian;
export type Galat = Partial<Record<NamaIsian | "consent", string>>;

export const ISIAN_KOSONG: Isian = {
  legal_name: "",
  nib: "",
  npwp: "",
  office_address: "",
  office_phone: "",
  hr_full_name: "",
  hr_job_title: "",
  hr_email: "",
};

/** Langkah formulir (penanda langkah wireframe K01). */
export const LANGKAH = ["Data perusahaan", "Data Anda", "Persetujuan dan kirim"] as const;
export type NomorLangkah = 0 | 1 | 2;

/** Field per langkah, urut tampil (dipakai untuk fokus ke galat pertama). */
export const ISIAN_LANGKAH: Record<0 | 1, readonly NamaIsian[]> = {
  0: ["legal_name", "nib", "npwp", "office_address", "office_phone"],
  1: ["hr_full_name", "hr_job_title", "hr_email"],
};

/** Label field, sama dengan yang dipakai di ringkasan. */
export const LABEL: Record<NamaIsian, string> = {
  legal_name: "Nama badan usaha",
  nib: "NIB",
  npwp: "NPWP perusahaan",
  office_address: "Alamat kantor",
  office_phone: "Telepon kantor",
  hr_full_name: "Nama lengkap",
  hr_job_title: "Jabatan",
  hr_email: "Email kerja",
};

/** Batas panjang dari kontrak `CorporateRegistrationRequest` (openapi/public.json). */
export const MAKS: Record<NamaIsian, number> = {
  legal_name: 200,
  nib: 13,
  npwp: 16,
  office_address: 500,
  office_phone: 32,
  hr_full_name: 120,
  hr_job_title: 100,
  hr_email: 254,
};

/** Petunjuk tetap di bawah field (Q6: NIB 13 angka, NPWP 16 angka; NPWP lama 15 angka ditambah 0 di depan). */
export const PETUNJUK: Partial<Record<NamaIsian, string>> = {
  nib: "13 angka, sesuai dokumen NIB.",
  npwp: "16 angka tanpa titik atau tanda hubung. NPWP lama berisi 15 angka: tambahkan angka 0 di depan.",
  hr_email: "Tautan aktivasi akun dikirim ke email ini.",
};

const WAJIB: Record<NamaIsian, string> = {
  legal_name: "Nama badan usaha wajib diisi.",
  nib: "NIB wajib diisi.",
  npwp: "NPWP wajib diisi.",
  office_address: "Alamat kantor wajib diisi.",
  office_phone: "Telepon kantor wajib diisi.",
  hr_full_name: "Nama lengkap wajib diisi.",
  hr_job_title: "Jabatan wajib diisi.",
  hr_email: "Email kerja wajib diisi.",
};

/** Galat satu field (dipakai juga formulir koreksi data perusahaan saat Ditolak, CR-036); `null` = sah. */
export function periksaSatu(nama: NamaIsian, nilaiMentah: string): string | null {
  const nilai = nilaiMentah.trim();
  if (nilai === "") return WAJIB[nama];
  if (nama === "nib" && !/^[0-9]{13}$/.test(nilai)) {
    return "Format NIB tidak sesuai. NIB berisi 13 angka; periksa kembali nomor di dokumen NIB Anda.";
  }
  if (nama === "npwp") {
    if (/^[0-9]{15}$/.test(nilai)) {
      return "NPWP berisi 15 angka (format lama). Tambahkan angka 0 di depan sehingga menjadi 16 angka.";
    }
    if (!/^[0-9]{16}$/.test(nilai)) return "Format NPWP tidak sesuai. Isi 16 angka tanpa titik atau tanda hubung.";
  }
  if (nilai.length > MAKS[nama]) return `${LABEL[nama]} paling banyak ${MAKS[nama].toString()} karakter.`;
  if (nama === "hr_email" && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(nilai)) {
    return "Format email tidak sesuai. Contoh: nama@perusahaan.co.id.";
  }
  return null;
}

/** Galat untuk satu langkah (0 atau 1); objek kosong = boleh lanjut. */
export function periksaLangkah(langkah: 0 | 1, isian: Isian): Galat {
  const galat: Galat = {};
  for (const nama of ISIAN_LANGKAH[langkah]) {
    const g = periksaSatu(nama, isian[nama]);
    if (g !== null) galat[nama] = g;
  }
  return galat;
}

/** Badan permintaan: nilai di-trim, tanpa field lain (kontrak `deny_unknown_fields`). */
export function badanPermintaan(isian: Isian, versiTeks: string, butir: readonly string[]) {
  return {
    legal_name: isian.legal_name.trim(),
    nib: isian.nib.trim(),
    npwp: isian.npwp.trim(),
    office_address: isian.office_address.trim(),
    office_phone: isian.office_phone.trim(),
    hr_full_name: isian.hr_full_name.trim(),
    hr_job_title: isian.hr_job_title.trim(),
    hr_email: isian.hr_email.trim(),
    consent_text_version_id: versiTeks,
    consent_item_ids: [...butir],
  };
}

/** Langkah tempat sebuah field berada (untuk galat 422 dari server). Field tak dikenal → langkah persetujuan. */
export function langkahDari(field: string): NomorLangkah {
  if ((ISIAN_LANGKAH[0] as readonly string[]).includes(field)) return 0;
  if ((ISIAN_LANGKAH[1] as readonly string[]).includes(field)) return 1;
  return 2;
}
