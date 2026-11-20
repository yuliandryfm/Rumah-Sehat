import { useId } from "react";
import type { InputHTMLAttributes } from "react";
import { cx } from "./cx.ts";

export interface CheckboxProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, "id" | "type" | "aria-invalid" | "defaultChecked"> {
  label: string;
  /** Pesan galat di bawah kotak (batas `danger` + teks). */
  error?: string;
}

/**
 * Kotak centang. State: default, hover, fokus, nonaktif, galat.
 * Kotak persetujuan tidak pernah tercentang otomatis (KEPUTUSAN-177): `defaultChecked` sengaja
 * tidak diterima; keadaan awal selalu tidak tercentang kecuali pemanggil mengendalikannya lewat
 * `checked` dari tindakan pengguna. Satu butir persetujuan satu kotak.
 */
export function Checkbox({ label, error, className, disabled, ...sisa }: CheckboxProps) {
  const id = useId();
  const errorId = `${id}-error`;
  const hasError = error !== undefined && error.length > 0;
  return (
    <div className={cx("rs-check", className)}>
      <label className="rs-check__row" htmlFor={id}>
        <input
          {...sisa}
          id={id}
          type="checkbox"
          className="rs-check__input"
          disabled={disabled === true}
          aria-invalid={hasError ? true : undefined}
          aria-describedby={hasError ? errorId : undefined}
        />
        <span className="rs-check__label">{label}</span>
      </label>
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
