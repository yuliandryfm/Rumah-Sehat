import type { AnchorHTMLAttributes, HTMLAttributes, ReactNode } from "react";
import { cx } from "./cx.ts";
import { Skeleton } from "./Skeleton.tsx";

export type CardTone = "standard" | "mint";

interface CardCommon {
  tone?: CardTone;
  /** Muatan sedang diambil: isi diganti kerangka dan kartu ditandai `aria-busy`. */
  loading?: boolean;
  children?: ReactNode;
}

export type StaticCardProps = CardCommon & Omit<HTMLAttributes<HTMLDivElement>, "children"> & { href?: undefined };
export type PressableCardProps = CardCommon &
  Omit<AnchorHTMLAttributes<HTMLAnchorElement>, "children"> & { href: string };
export type CardProps = StaticCardProps | PressableCardProps;

/**
 * Kartu. Tanpa `href` kartu statis (`elevation.1`); dengan `href` kartu dapat ditekan (tautan berlabel
 * jelas) dengan state hover dan fokus. Saat `loading`, isi diganti kerangka.
 */
export function Card(props: CardProps) {
  const { tone = "standard", loading = false, children, className, ...sisa } = props;
  const kelas = cx("rs-card", tone === "mint" && "rs-card--mint", props.href !== undefined && "rs-card--pressable", className);
  const isi = loading ? <Skeleton lines={3} /> : children;
  if (props.href !== undefined) {
    const { href, ...anchor } = sisa as Omit<PressableCardProps, keyof CardCommon | "className">;
    return (
      <a {...anchor} href={href} className={kelas} aria-busy={loading ? true : undefined}>
        {isi}
      </a>
    );
  }
  return (
    <div {...(sisa as Omit<StaticCardProps, keyof CardCommon | "className">)} className={kelas} aria-busy={loading ? true : undefined}>
      {isi}
    </div>
  );
}
