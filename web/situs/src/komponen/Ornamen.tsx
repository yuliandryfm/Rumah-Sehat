/**
 * Ornamen dekoratif situs (CR-033 C-01, konsep A "Ruang Teduh", brief §7.2): bentuk daun dan blob organik
 * sebagai SVG sebaris di kode, bukan gambar eksternal. Semua `aria-hidden` dan tanpa teks; warna dari kelas
 * CSS (token), bukan atribut. Tidak membawa informasi, sehingga tidak tunduk syarat kontras non-teks.
 */

const TITIK: readonly (readonly [number, number])[] = Array.from({ length: 25 }, (_, i) => [8 + (i % 5) * 18, 8 + Math.floor(i / 5) * 18] as const);

/** Blob daun besar di belakang visual hero Beranda, ditambah daun aksen dan kisi titik. */
export function OrnamenHero() {
  return (
    <svg className="situs-orn situs-orn--hero" viewBox="0 0 520 520" aria-hidden="true" focusable="false">
      <path className="situs-orn__blob" d="M262 34c104-6 205 58 220 168 15 112-48 236-170 270-118 33-245-25-272-140C13 218 96 44 262 34z" />
      <path className="situs-orn__daun" d="M70 470C52 352 128 238 300 222c14 170-82 262-230 248z" />
      <path className="situs-orn__urat" d="M78 462c50-70 110-130 196-210" />
      <g className="situs-orn__titik" transform="translate(410 20)">
        {TITIK.map(([x, y]) => (
          <circle key={`${String(x)}-${String(y)}`} cx={x} cy={y} r="2.5" />
        ))}
      </g>
    </svg>
  );
}

/** Lengkung lembut di sudut hero halaman isi. */
export function OrnamenHalaman() {
  return (
    <svg className="situs-orn situs-orn--halaman" viewBox="0 0 400 300" aria-hidden="true" focusable="false">
      <path className="situs-orn__blob" d="M400 0v300c-70-10-140-60-170-130C200 100 250 20 320 0z" />
      <path className="situs-orn__daun" d="M250 290c-10-80 40-150 140-160 6 100-50 168-140 160z" />
    </svg>
  );
}

/** Cincin konsentris untuk seksi gelap (Untuk Perusahaan). */
export function OrnamenGelap() {
  return (
    <svg className="situs-orn situs-orn--gelap" viewBox="0 0 480 480" aria-hidden="true" focusable="false">
      {[220, 170, 120, 70].map((r) => (
        <circle key={r} className="situs-orn__cincin" cx="240" cy="240" r={r} />
      ))}
      <path className="situs-orn__daun" d="M150 380c-12-96 50-180 170-192 8 120-60 202-170 192z" />
    </svg>
  );
}
