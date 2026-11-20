import type { HTMLAttributes } from "react";
import { cx } from "./cx.ts";

export type BadgeTone = "info" | "success" | "warning" | "danger" | "neutral";

/** Simbol per nada (design guideline §5.6, KEPUTUSAN-169): status tidak hanya dibedakan oleh warna. */
export const BADGE_SYMBOL: Record<BadgeTone, string> = {
  info: "●",
  success: "✓",
  warning: "!",
  danger: "✕",
  neutral: "○",
};

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  tone: BadgeTone;
  /** Nama status persis seperti di PRD §3.6 (P3). */
  children: string;
}

/** Badge status: simbol + teks nama status. Hanya satu state (default). */
export function Badge({ tone, className, children, ...sisa }: BadgeProps) {
  return (
    <span {...sisa} className={cx("rs-badge", `rs-badge--${tone}`, className)}>
      <span aria-hidden="true" className="rs-badge__symbol">
        {BADGE_SYMBOL[tone]}
      </span>
      <span>{children}</span>
    </span>
  );
}
