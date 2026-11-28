import type { ReactNode } from "react";
import { gambarDiizinkan } from "../konten/aturan-aman.ts";
import { pisahPembuka } from "../konten/struktur.ts";
import type { Konten } from "../konten/tipe.ts";
import { BlokMarkdown, Markdown } from "./Markdown.tsx";
import { OrnamenHalaman } from "./Ornamen.tsx";
import { TidakDiketahui } from "./TidakDiketahui.tsx";

/** Tanggal tayang dalam zona Asia/Jakarta (WIB); `null` bila tidak dapat dibaca. */
export function formatTanggal(iso: string): string | null {
  const t = new Date(iso);
  if (Number.isNaN(t.getTime())) return null;
  return new Intl.DateTimeFormat("id-ID", {
    dateStyle: "long",
    timeZone: "Asia/Jakarta",
  }).format(t);
}

/** Gambar konten layak tampil: hanya dari endpoint gambar publik dan bertekstual alternatif (US-1802 AC5). */
export function adaGambar(konten: Konten | null): boolean {
  const g = konten?.image;
  return g !== null && g !== undefined && gambarDiizinkan(g.url) && g.alt_text.trim() !== "";
}

/** Gambar konten: hanya dari endpoint gambar publik backend, wajib teks alternatif (US-1802 AC5, SIT-01). */
export function GambarKonten({ konten }: { konten: Konten }) {
  const g = konten.image;
  if (g === null || g === undefined || !adaGambar(konten)) return null;
  return <img className="situs-gambar-konten" src={g.url} alt={g.alt_text} />;
}

function PenandaIsiKosong() {
  return (
    <p>
      [Isi halaman dari CMS.] <TidakDiketahui />
    </p>
  );
}

/** Isi (Markdown) atau penanda `[TIDAK DIKETAHUI]` bila konten tidak punya isi. */
export function IsiKonten({ konten }: { konten: Konten }) {
  if (konten.body === null || konten.body === undefined || konten.body.trim() === "") return <PenandaIsiKosong />;
  return <Markdown teks={konten.body} />;
}

/**
 * Pita judul halaman (CR-042, desain Stitch): lencana pil (label), judul display, kalimat pembuka; latar rata `bg.app`
 * dengan ornamen lengkung mint di kanan (tanpa varian gelap). Teks `text.primary` 14,14; label `primary.strong` di atas mint.
 */
export function HeroHalaman({ judul, label, children }: { judul: string; label?: string; children?: ReactNode }) {
  return (
    <div className="situs-halhero">
      <OrnamenHalaman />
      <div className="situs-wadah situs-halhero__isi">
        {label === undefined ? null : <p className="situs-eyebrow">{label}</p>}
        <h1>{judul}</h1>
        {children}
      </div>
    </div>
  );
}

/**
 * Halaman tetap dari CMS (Tentang Kami, Layanan, Untuk Perusahaan, Rekanan, Kontak, Unduh aplikasi,
 * Kebijakan). Paragraf pembuka Markdown (bila isi diawali paragraf) menjadi kalimat pembuka di pita judul;
 * sisanya dirender di kolom baca. `draf` menampilkan penanda draf serta tanggal berlaku dan versi
 * (US-1801 AC4).
 */
export function HalamanKonten({ konten, draf = false, label }: { konten: Konten; draf?: boolean; label?: string }) {
  const berlaku = formatTanggal(konten.published_at);
  // Draf (Kebijakan, US-1801 AC4): paragraf pembuka tidak dipindah ke pita judul, sehingga spanduk "Draf."
  // tampil sebelum teks isi pertama.
  const terpisah = pisahPembuka(konten.body);
  const bagian = draf && terpisah !== null ? { pembuka: null, sisa: terpisah.pembuka === null ? terpisah.sisa : [terpisah.pembuka, ...terpisah.sisa] } : terpisah;
  const pembuka = bagian?.pembuka ?? null;
  const kosongSesudahPembuka = bagian !== null && bagian.sisa.length === 0;
  return (
    <article className="situs-halaman">
      <HeroHalaman judul={konten.title} {...(label === undefined ? {} : { label })}>
        {pembuka === null ? null : <BlokMarkdown nodes={[pembuka]} kelas="situs-md situs-lead" />}
      </HeroHalaman>
      {draf || adaGambar(konten) || !kosongSesudahPembuka ? (
        <div className="situs-sec situs-sec--rapat">
          <div className="situs-baca">
            {draf ? (
              <div className="situs-draf" role="note">
                <p>
                  <strong>Draf.</strong> Teks final menunggu keputusan hukum dan belum berlaku sebagai ketentuan akhir.
                </p>
                <p className="situs-kecil">
                  Berlaku sejak: {berlaku ?? <TidakDiketahui />}. Versi: <TidakDiketahui />
                </p>
              </div>
            ) : null}
            {adaGambar(konten) ? (
              <div className="situs-bingkai">
                <GambarKonten konten={konten} />
              </div>
            ) : null}
            {bagian === null ? <PenandaIsiKosong /> : <BlokMarkdown nodes={bagian.sisa} />}
          </div>
        </div>
      ) : null}
    </article>
  );
}
