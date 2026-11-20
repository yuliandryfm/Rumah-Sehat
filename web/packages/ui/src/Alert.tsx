import type { HTMLAttributes, ReactNode } from "react";
import { BADGE_SYMBOL } from "./Badge.tsx";
import { cx } from "./cx.ts";

export type AlertVariant = "info" | "success" | "warning" | "danger";

export interface AlertProps extends Omit<HTMLAttributes<HTMLDivElement>, "title"> {
  variant: AlertVariant;
  title: string;
  children?: ReactNode;
}

/**
 * Peringatan: latar status, garis kiri 4 px, ikon, judul tebal, teks `text.primary` (K12).
 * Diumumkan sebagai pesan status (SC 4.1.3): `role="alert"` untuk bahaya, `role="status"` selain itu.
 */
export function Alert({ variant, title, className, children, ...sisa }: AlertProps) {
  return (
    <div
      role={variant === "danger" ? "alert" : "status"}
      {...sisa}
      className={cx("rs-alert", `rs-alert--${variant}`, className)}
    >
      <span aria-hidden="true" className="rs-alert__icon">
        {BADGE_SYMBOL[variant]}
      </span>
      <div className="rs-alert__body">
        <p className="rs-alert__title">{title}</p>
        {children === undefined ? null : <div className="rs-alert__text">{children}</div>}
      </div>
    </div>
  );
}
