/**
 * Logo Ruang Sehat (lambang rumah-daun) sebagai SVG sebaris: berkas sumber dari paket desain user
 * (CR-042; salinan di `docs/design/aset/logo-ruang-sehat.svg`, jalur identik, diperiksa di logo.test.tsx).
 * Sebaris supaya tidak ada berkas statis tambahan atau sumber gambar baru di CSP. Dekoratif (`aria-hidden`):
 * nama merek selalu ada sebagai teks di sebelahnya. Warna lambang adalah aset merek (bukan token teks).
 */
export const JALUR_LOGO = {
  rumah:
    "M32 6C29.8 6 27.7 6.7 26 8L13.5 17.5C11.3 19.2 10 21.8 10 24.6V48C10 51.3 12.7 54 16 54H33.5C30.2 51.5 28.5 47.4 29 43H20V26.2C20 24.6 20.8 23.1 22.1 22.1L29.6 16.4C31 15.3 33 15.3 34.4 16.4L41.9 22.1C43.2 23.1 44 24.6 44 26.2V31.5C47.7 28.6 51.4 26.8 54 26.2V24.6C54 21.8 52.7 19.2 50.5 17.5L38 8C36.3 6.7 34.2 6 32 6Z",
  daun: "M55.5 28.5C44.5 29 35.5 36.2 33.8 46.8C33.3 49.8 33.7 52.5 34.6 54.5C38.5 54.8 43.2 53.7 47.3 50.6C53.2 46.1 56.2 38.2 55.5 28.5Z",
  urat: "M35.2 53.5C38.8 46.8 44.2 41.2 51.2 36.2",
} as const;

export function Logo({ kelas = "situs-logo__tanda" }: { kelas?: string }) {
  return (
    <svg className={kelas} viewBox="0 0 64 64" fill="none" aria-hidden="true" focusable="false">
      <path d={JALUR_LOGO.rumah} fill="#087F78" />
      <path d={JALUR_LOGO.daun} fill="#2AA188" />
      <path d={JALUR_LOGO.urat} stroke="#FFFFFF" strokeWidth="2.4" strokeLinecap="round" />
    </svg>
  );
}
