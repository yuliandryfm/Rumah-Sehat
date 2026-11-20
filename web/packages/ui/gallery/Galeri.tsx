import type { ReactNode } from "react";
import { Alert, Badge, Button, Card, Checkbox, Skeleton, TextField } from "../src/index.ts";
import type { AlertVariant, BadgeTone, ButtonVariant, KomponenName, StateName } from "../src/index.ts";

/**
 * Halaman galeri state komponen dasar (design guideline §4). Setiap sel punya
 * `data-galeri="komponen:state"` yang dicocokkan test dengan `STATE_KOMPONEN`. State hover dan fokus
 * dipaksa lewat `data-state` supaya terlihat tanpa interaksi; data di sini sintetis.
 */

function Sel({ komponen, state, judul, children }: { komponen: KomponenName; state: StateName; judul: string; children: ReactNode }) {
  return (
    <div className="galeri__sel" data-galeri={`${komponen}:${state}`}>
      <p className="galeri__judul-sel">{judul}</p>
      {children}
    </div>
  );
}

const VARIAN_TOMBOL: { variant: ButtonVariant; label: string }[] = [
  { variant: "primary", label: "Bayar Rp150.000" },
  { variant: "secondary", label: "Lihat rincian" },
  { variant: "text", label: "Batalkan" },
  { variant: "destructive", label: "Tarik persetujuan" },
];

const NADA_BADGE: { tone: BadgeTone; nama: string }[] = [
  { tone: "info", nama: "Terkonfirmasi" },
  { tone: "success", nama: "Selesai" },
  { tone: "warning", nama: "Menunggu pembayaran" },
  { tone: "danger", nama: "Ditolak" },
  { tone: "neutral", nama: "Kedaluwarsa" },
];

const VARIAN_ALERT: { variant: AlertVariant; judul: string; isi: string }[] = [
  { variant: "info", judul: "Slot ditahan", isi: "Selesaikan pembayaran sebelum 10.15 WIB." },
  { variant: "success", judul: "Pembayaran berhasil", isi: "Pesanan Anda terkonfirmasi." },
  { variant: "warning", judul: "Sisa waktu 5 menit", isi: "Slot akan dilepas setelah waktu habis." },
  { variant: "danger", judul: "Gagal terhubung", isi: "Periksa koneksi Anda lalu coba lagi." },
];

export function Galeri() {
  return (
    <main className="galeri">
      <h1 className="galeri__judul">Galeri state komponen</h1>
      <p>Komponen dasar Fase 1 dengan semua state (docs/06-design-guideline.md §4). Data sintetis.</p>

      <section aria-labelledby="g-button">
        <h2 id="g-button">Tombol</h2>
        {VARIAN_TOMBOL.map(({ variant, label }) => (
          <div className="galeri__baris" key={variant}>
            <Sel komponen="button" state="default" judul={`${variant}: default`}>
              <Button variant={variant}>{label}</Button>
            </Sel>
            <Sel komponen="button" state="hover" judul={`${variant}: hover`}>
              <Button variant={variant} data-state="hover">
                {label}
              </Button>
            </Sel>
            <Sel komponen="button" state="fokus" judul={`${variant}: fokus`}>
              <Button variant={variant} data-state="fokus">
                {label}
              </Button>
            </Sel>
            <Sel komponen="button" state="nonaktif" judul={`${variant}: nonaktif`}>
              <Button variant={variant} disabled>
                {label}
              </Button>
            </Sel>
            <Sel komponen="button" state="memuat" judul={`${variant}: memuat`}>
              <Button variant={variant} loading>
                {label}
              </Button>
            </Sel>
          </div>
        ))}
      </section>

      <section aria-labelledby="g-textfield">
        <h2 id="g-textfield">Isian teks</h2>
        <div className="galeri__baris">
          <Sel komponen="textfield" state="default" judul="default">
            <TextField label="Nama lengkap" hint="Sesuai kartu identitas." />
          </Sel>
          <Sel komponen="textfield" state="hover" judul="hover">
            <TextField label="Nama lengkap" data-state="hover" />
          </Sel>
          <Sel komponen="textfield" state="fokus" judul="fokus">
            <TextField label="Nama lengkap" data-state="fokus" />
          </Sel>
          <Sel komponen="textfield" state="nonaktif" judul="nonaktif">
            <TextField label="Nama lengkap" disabled defaultValue="Tidak dapat diubah" />
          </Sel>
          <Sel komponen="textfield" state="galat" judul="galat">
            <TextField label="Nomor telepon" error="Nomor telepon harus diawali 08 atau +62." defaultValue="12" />
          </Sel>
        </div>
      </section>

      <section aria-labelledby="g-checkbox">
        <h2 id="g-checkbox">Checkbox</h2>
        <div className="galeri__baris">
          <Sel komponen="checkbox" state="default" judul="default (tidak tercentang otomatis)">
            <Checkbox label="Saya menyetujui pemrosesan data sesuai kebijakan privasi." />
          </Sel>
          <Sel komponen="checkbox" state="hover" judul="hover">
            <Checkbox label="Saya menyetujui pemrosesan data." data-state="hover" />
          </Sel>
          <Sel komponen="checkbox" state="fokus" judul="fokus">
            <Checkbox label="Saya menyetujui pemrosesan data." data-state="fokus" />
          </Sel>
          <Sel komponen="checkbox" state="nonaktif" judul="nonaktif">
            <Checkbox label="Pilihan tidak tersedia." disabled />
          </Sel>
          <Sel komponen="checkbox" state="galat" judul="galat">
            <Checkbox label="Saya menyetujui syarat layanan." error="Centang kotak ini untuk melanjutkan." />
          </Sel>
        </div>
      </section>

      <section aria-labelledby="g-card">
        <h2 id="g-card">Kartu</h2>
        <div className="galeri__baris">
          <Sel komponen="card" state="default" judul="default">
            <Card>Riwayat konsultasi terakhir</Card>
          </Sel>
          <Sel komponen="card" state="hover" judul="hover (dapat ditekan)">
            <Card href="#kartu" data-state="hover">
              Lihat pesanan
            </Card>
          </Sel>
          <Sel komponen="card" state="fokus" judul="fokus (dapat ditekan)">
            <Card href="#kartu" data-state="fokus">
              Lihat pesanan
            </Card>
          </Sel>
          <Sel komponen="card" state="memuat" judul="memuat (kerangka)">
            <Card loading>Tidak tampil saat memuat</Card>
          </Sel>
        </div>
      </section>

      <section aria-labelledby="g-badge">
        <h2 id="g-badge">Badge status</h2>
        <Sel komponen="badge" state="default" judul="lima nada: simbol + nama status">
          <div className="galeri__baris">
            {NADA_BADGE.map(({ tone, nama }) => (
              <Badge key={tone} tone={tone}>
                {nama}
              </Badge>
            ))}
          </div>
        </Sel>
      </section>

      <section aria-labelledby="g-alert">
        <h2 id="g-alert">Peringatan</h2>
        <Sel komponen="alert" state="default" judul="empat varian">
          <div className="galeri__kolom">
            {VARIAN_ALERT.map(({ variant, judul, isi }) => (
              <Alert key={variant} variant={variant} title={judul}>
                {isi}
              </Alert>
            ))}
          </div>
        </Sel>
      </section>

      <section aria-labelledby="g-skeleton">
        <h2 id="g-skeleton">Kerangka</h2>
        <Sel komponen="skeleton" state="default" judul="tiga baris">
          <Skeleton lines={3} />
        </Sel>
      </section>
    </main>
  );
}
