import type { GambarKartu, NamaIkonLayanan } from "./struktur.ts";

/**
 * Foto kartu layanan per ikon (slot foto, TD-69, KEPUTUSAN-481): kosong sampai user menyerahkan berkas gambar
 * (13 foto dari paket desain tidak dapat diunduh karena URL meminta masuk akun Google). Saat berkas ada di
 * `web/situs/public/foto/`, tambahkan entri di sini, mis. `konsultasi: { src: "/foto/konsultasi-dokter.webp",
 * alt: "..." }`; seluruh kartu (Beranda dan Layanan) memakainya tanpa perubahan lain.
 */
export const FOTO_LAYANAN: Readonly<Partial<Record<NamaIkonLayanan, GambarKartu>>> = {};
