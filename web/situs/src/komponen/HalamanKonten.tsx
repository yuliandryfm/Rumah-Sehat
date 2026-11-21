import { gambarDiizinkan } from "../konten/aturan-aman.ts";
import type { Konten } from "../konten/tipe.ts";
import { Markdown } from "./Markdown.tsx";
import { TidakDiketahui } from "./TidakDiketahui.tsx";

/** Tanggal tayang dalam zona Asia/Jakarta (WIB); `null` bila tidak dapat dibaca. */
export function formatTanggal(iso: string): string | null {
  const t = new Date(iso);
  if (Number.isNaN(t.getTime())) return null;
  return new Intl.DateTimeFormat("id-ID", { dateStyle: "long", timeZone: "Asia/Jakarta" }).format(t);
}

/** Gambar konten: hanya dari endpoint gambar publik backend, wajib teks alternatif (US-1802 AC5, SIT-01). */
export function GambarKonten({ konten }: { konten: Konten }) {
  const g = konten.image;
  if (g === null || g === undefined || !gambarDiizinkan(g.url) || g.alt_text.trim() === "") return null;
  return <img className="situs-gambar-konten" src={g.url} alt={g.alt_text} />;
}

/** Isi (Markdown) atau penanda `[TIDAK DIKETAHUI]` bila konten tidak punya isi. */
export function IsiKonten({ konten }: { konten: Konten }) {
  if (konten.body === null || konten.body === undefined || konten.body.trim() === "") {
    return (
      <p>
        [Isi halaman dari CMS.] <TidakDiketahui />
      </p>
    );
  }
  return <Markdown teks={konten.body} />;
}

/**
 * Halaman tetap dari CMS (Tentang Kami, Layanan, Untuk Perusahaan, Rekanan, Kontak, Unduh aplikasi,
 * Kebijakan). `draf` menampilkan penanda draf serta tanggal berlaku dan versi (US-1801 AC4).
 */
export function HalamanKonten({ konten, draf = false }: { konten: Konten; draf?: boolean }) {
  const berlaku = formatTanggal(konten.published_at);
  return (
    <article className="situs-sec situs-baca">
      <h1>{konten.title}</h1>
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
      <GambarKonten konten={konten} />
      <IsiKonten konten={konten} />
    </article>
  );
}
