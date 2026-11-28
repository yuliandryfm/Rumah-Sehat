import { fotoLokalDiizinkan } from "../konten/aturan-aman.ts";
import type { KartuLayanan } from "../konten/struktur.ts";
import { IkonLingkar } from "./Ikon.tsx";

/**
 * Panel gambar kartu layanan (Beranda dan Layanan): `<img>` bila kartu punya `gambar` dengan jalur lokal yang
 * lolos `fotoLokalDiizinkan` (TD-69); selain itu ilustrasi ikon dekoratif. Slot terisi lewat `konten/foto.ts`.
 */
export function PanelKartu({ kartu }: { kartu: Pick<KartuLayanan, "ikon" | "gambar"> }) {
  if (kartu.gambar === undefined || !fotoLokalDiizinkan(kartu.gambar.src) || kartu.gambar.alt.trim() === "") return <IkonLingkar nama={kartu.ikon} />;
  return (
    <span className="situs-layanan__foto">
      <img src={kartu.gambar.src} alt={kartu.gambar.alt} loading="lazy" />
    </span>
  );
}
