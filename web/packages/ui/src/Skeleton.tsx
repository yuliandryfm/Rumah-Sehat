import { cx } from "./cx.ts";

export interface SkeletonProps {
  /** Jumlah baris kerangka. Bawaan 1. */
  lines?: number;
  className?: string;
}

/**
 * Kerangka (skeleton) untuk muatan > 300 ms (design guideline §5.1). Tanpa animasi berkedip;
 * teks tersembunyi "Memuat" untuk pembaca layar. Hanya satu pengumuman per kelompok.
 */
export function Skeleton({ lines = 1, className }: SkeletonProps) {
  const jumlah = Math.max(1, Math.floor(lines));
  return (
    <div role="status" className={cx("rs-skeleton", className)}>
      <span className="rs-sr-only">Memuat</span>
      {Array.from({ length: jumlah }, (_, i) => (
        <span key={i} aria-hidden="true" className="rs-skeleton__bar" />
      ))}
    </div>
  );
}
