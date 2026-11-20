import { useId } from "react";
import type { InputHTMLAttributes } from "react";
import { cx } from "./cx.ts";

export interface TextFieldProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "id" | "aria-invalid"> {
  /** Label selalu terlihat; bukan placeholder saja (design guideline §4). */
  label: string;
  /** Teks bantuan di bawah isian. */
  hint?: string;
  /**
   * Pesan galat dari `errors[].message` respons API (ADR-023). Harus menyebut cara memperbaiki
   * (SC 3.3.1). Ada nilai berarti isian galat.
   */
  error?: string;
}

/**
 * Isian teks satu baris. State: default, hover, fokus, nonaktif, galat. Galat ditandai batas 2 px,
 * ikon "!" dan teks (bukan warna saja, A1), dan dihubungkan ke isian lewat `aria-describedby`.
 * Paste tidak pernah diblokir (A14).
 */
export function TextField({ label, hint, error, className, disabled, ...sisa }: TextFieldProps) {
  const id = useId();
  const hintId = `${id}-hint`;
  const errorId = `${id}-error`;
  const hasError = error !== undefined && error.length > 0;
  const hasHint = hint !== undefined && hint.length > 0;
  const describedBy = [hasHint ? hintId : undefined, hasError ? errorId : undefined].filter(Boolean).join(" ");
  return (
    <div className={cx("rs-field", className)}>
      <label className="rs-field__label" htmlFor={id}>
        {label}
      </label>
      <input
        {...sisa}
        id={id}
        className="rs-field__input"
        disabled={disabled === true}
        aria-invalid={hasError ? true : undefined}
        aria-describedby={describedBy.length > 0 ? describedBy : undefined}
      />
      {hasHint ? (
        <p id={hintId} className="rs-field__hint">
          {hint}
        </p>
      ) : null}
      {hasError ? (
        <p id={errorId} className="rs-field__error">
          <span aria-hidden="true" className="rs-field__error-icon">
            !
          </span>
          <span>{error}</span>
        </p>
      ) : null}
    </div>
  );
}
