import type { ButtonHTMLAttributes, MouseEvent } from "react";
import { cx } from "./cx.ts";

export type ButtonVariant = "primary" | "secondary" | "text" | "destructive";

export interface ButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "type"> {
  /** Satu tombol utama per layar (design guideline §4). Bawaan: `primary`. */
  variant?: ButtonVariant;
  /**
   * Permintaan sedang berjalan: tombol menampilkan pemutar + label tetap dan terkunci sampai respons
   * (KU-05, P5). Terkunci lewat `aria-disabled`, bukan `disabled`, supaya fokus tidak hilang.
   */
  loading?: boolean;
  /** Label saat memuat. Bawaan "Memproses...". */
  loadingLabel?: string;
  type?: "button" | "submit" | "reset";
}

/**
 * Tombol. State: default, hover, fokus, nonaktif, memuat (design guideline §4).
 * Tombol utama tidak dinonaktifkan karena validasi isian (KEPUTUSAN-168): `disabled` hanya untuk aksi
 * yang memang tidak tersedia, dan alasannya harus tertulis di dekat tombol oleh pemanggil.
 */
export function Button({
  variant = "primary",
  loading = false,
  loadingLabel = "Memproses...",
  type = "button",
  className,
  children,
  onClick,
  disabled,
  ...sisa
}: ButtonProps) {
  const tekan = (e: MouseEvent<HTMLButtonElement>) => {
    if (loading) {
      e.preventDefault();
      return;
    }
    onClick?.(e);
  };
  return (
    <button
      {...sisa}
      type={type}
      disabled={disabled === true}
      aria-disabled={loading ? true : undefined}
      aria-busy={loading ? true : undefined}
      data-loading={loading ? "true" : undefined}
      className={cx("rs-btn", `rs-btn--${variant}`, className)}
      onClick={tekan}
    >
      {loading ? (
        <>
          <span className="rs-spinner" aria-hidden="true" />
          <span>{loadingLabel}</span>
        </>
      ) : (
        children
      )}
    </button>
  );
}
