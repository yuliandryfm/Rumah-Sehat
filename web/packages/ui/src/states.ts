/**
 * Matriks state komponen Fase 1 menurut design guideline §4 (kolom Default, Hover, Fokus, Nonaktif,
 * Galat, Memuat). Dipakai halaman galeri (`data-galeri="komponen:state"`) dan test yang memastikan
 * setiap state ada di galeri dan punya aturan CSS. Menambah state di sini tanpa menampilkannya di
 * galeri membuat test gagal.
 */
export type StateName = "default" | "hover" | "fokus" | "nonaktif" | "galat" | "memuat";

export const STATE_KOMPONEN = {
  button: ["default", "hover", "fokus", "nonaktif", "memuat"],
  textfield: ["default", "hover", "fokus", "nonaktif", "galat"],
  checkbox: ["default", "hover", "fokus", "nonaktif", "galat"],
  card: ["default", "hover", "fokus", "memuat"],
  badge: ["default"],
  alert: ["default"],
  skeleton: ["default"],
} as const satisfies Record<string, readonly StateName[]>;

export type KomponenName = keyof typeof STATE_KOMPONEN;
