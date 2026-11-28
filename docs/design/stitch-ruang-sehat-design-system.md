---
name: "Ruang Sehat — Vitality & Trust"
description: "Modern editorial health-lifestyle design system combining deep botanical emerald-teal, warm terracotta accents, and crisp botanical neutrals."
---

## Overview

Ruang Sehat is an Indonesian health-lifestyle and telemedicine platform operated by PT. Arunika. The design system balances clinical trustworthiness with dynamic, modern lifestyle energy. Rather than sterile hospital white and rigid wireframes, the interface pairs fresh botanical off-white (`#FAFBF7`), deep brand teal (`#087F78`), and soft mint-teal tints (`#DFF4ED`) with editorial photography and rounded, approachable geometry.

## Colors

The palette is anchored in the brand's signature deep teal (`#087F78` primary, `#00645E` primary-container) that conveys medical reliability and calm vitality, paired with harmonious botanical teal-mint accents (`#006A61` secondary, `#DFF4ED` mint tint/soft container) for badges, highlights, and interactive accents. Surfaces sit on a fresh off-white (`#FAFBF7`) with pure white (`#FFFFFF`) elevated cards.

- **Primary:** Deep brand teal (`#087F78`) for primary CTAs, active navigation states, and corporate wellness banners.
- **Secondary:** Harmonious botanical teal (`#006A61`) paired with soft mint (`#DFF4ED`) for lifestyle badges, icon containers, and subtle highlights.
- **Surface & Containers:** Clean botanical off-white (`#FAFBF7`) stepping to pure white (`#FFFFFF`) elevated cards and soft cool-mint neutrals (`#F0F4F3`).

```yaml
colors:
  primary-soft: "#B1FFF7"
  primary-tint: "#DFF4ED"
  secondary-soft: "#DFF4ED"
  success: "#087F78"
  on-success: "#FFFFFF"
  light:
    primary: "#00645E"
    on-primary: "#FFFFFF"
    primary-container: "#00504B"
    on-primary-container: "#D9FFFA"
    secondary: "#006A61"
    on-secondary: "#FFFFFF"
    secondary-container: "#92F4E5"
    on-secondary-container: "#005048"
    tertiary: "#8A4627"
    on-tertiary: "#FFFFFF"
    tertiary-container: "#A85E3D"
    on-tertiary-container: "#FFF6F3"
    error: "#BA1A1A"
    on-error: "#FFFFFF"
    error-container: "#FFDAD6"
    on-error-container: "#93000A"
    background: "#FAFBF7"
    on-background: "#181C1C"
    surface: "#FAFBF7"
    surface-dim: "#D7DBD9"
    surface-bright: "#FFFFFF"
    surface-container-lowest: "#FFFFFF"
    surface-container-low: "#F0F4F3"
    surface-container: "#EBEFED"
    surface-container-high: "#E5E9E7"
    surface-container-highest: "#DFE3E2"
    on-surface: "#181C1C"
    surface-variant: "#D9E5E3"
    on-surface-variant: "#3E4947"
    outline: "#636E6D"
    outline-variant: "#BDC9C7"
    inverse-surface: "#2D3131"
    inverse-on-surface: "#EEF2F0"
    inverse-primary: "#78D6CE"
  dark:
    primary: "#78D6CE"
    on-primary: "#003734"
    primary-container: "#087F78"
    on-primary-container: "#D9FFFA"
    secondary: "#75D7C9"
    on-secondary: "#003732"
    secondary-container: "#006A61"
    on-secondary-container: "#DFFFF8"
    tertiary: "#FFB597"
    on-tertiary: "#561F03"
    tertiary-container: "#A85E3D"
    on-tertiary-container: "#FFF6F3"
    error: "#FFB4AB"
    on-error: "#690005"
    error-container: "#93000A"
    on-error-container: "#FFDAD6"
    background: "#101414"
    on-background: "#DFE3E2"
    surface: "#101414"
    surface-dim: "#101414"
    surface-bright: "#353A39"
    surface-container-lowest: "#0A0F0E"
    surface-container-low: "#181C1C"
    surface-container: "#1C2120"
    surface-container-high: "#262B2A"
    surface-container-highest: "#313635"
    on-surface: "#DFE3E2"
    surface-variant: "#3E4947"
    on-surface-variant: "#BDC9C7"
    outline: "#879391"
    outline-variant: "#3E4947"
    inverse-surface: "#DFE3E2"
    inverse-on-surface: "#2D3131"
    inverse-primary: "#087F78"
```

## Typography

**Instrument Sans** drives display and section headlines with crisp geometric clarity and modern editorial authority. **Alata** handles body copy and UI labels with approachable warmth and clean legibility. **Spline Sans Mono** is reserved for tabular metrics, pricing figures, and timestamps.

```yaml
typography:
  display-lg:
    fontFamily: "Instrument Sans"
    fontSize: 56px
    fontWeight: 700
    lineHeight: 62px
    letterSpacing: -1.2px
  display-lg-mobile:
    fontFamily: "Instrument Sans"
    fontSize: 32px
    fontWeight: 700
    lineHeight: 38px
    letterSpacing: -0.6px
  headline-lg:
    fontFamily: "Instrument Sans"
    fontSize: 40px
    fontWeight: 700
    lineHeight: 48px
    letterSpacing: -0.8px
  headline-lg-mobile:
    fontFamily: "Instrument Sans"
    fontSize: 28px
    fontWeight: 700
    lineHeight: 34px
    letterSpacing: -0.4px
  headline-md:
    fontFamily: "Instrument Sans"
    fontSize: 28px
    fontWeight: 600
    lineHeight: 36px
    letterSpacing: -0.4px
  headline-sm:
    fontFamily: "Instrument Sans"
    fontSize: 22px
    fontWeight: 600
    lineHeight: 28px
    letterSpacing: -0.2px
  title-lg:
    fontFamily: "Instrument Sans"
    fontSize: 18px
    fontWeight: 600
    lineHeight: 26px
    letterSpacing: 0px
  body-lg:
    fontFamily: "Alata"
    fontSize: 17px
    fontWeight: 400
    lineHeight: 28px
    letterSpacing: 0px
  body-md:
    fontFamily: "Alata"
    fontSize: 15px
    fontWeight: 400
    lineHeight: 24px
    letterSpacing: 0px
  label-lg:
    fontFamily: "Instrument Sans"
    fontSize: 14px
    fontWeight: 600
    lineHeight: 20px
    letterSpacing: 0.1px
  label-sm:
    fontFamily: "Instrument Sans"
    fontSize: 12px
    fontWeight: 600
    lineHeight: 16px
    letterSpacing: 0.4px
  mono-md:
    fontFamily: "Spline Sans Mono"
    fontSize: 14px
    fontWeight: 500
    lineHeight: 20px
    letterSpacing: 0px
```

## Layout & Spacing

The desktop experience uses a centered max-width container (`1280px`) with a 12-column CSS Grid, generous `24px`–`32px` grid gutters, and `72px`–`96px` vertical rhythm between editorial sections. Mobile collapses gracefully to a single column with `20px` horizontal padding.

```yaml
spacing:
  gutter-xs: 4px
  gutter-sm: 8px
  gutter-md: 16px
  gutter-lg: 24px
  gutter-xl: 32px
  section: 72px
  page-margin: 24px
```

## Elevation & Depth

Hierarchy relies on tonal surface stepping (`surface` → `surface-container-lowest` `#FFFFFF` cards) paired with subtle `1px` `outline-variant` hairlines and soft emerald-tinted ambient shadows on floating UI previews. Avoid heavy drop shadows or stacked borders.

## Shapes

Generous, friendly corner radii reinforce the approachable lifestyle-health personality: pill-shaped (`full`) buttons and category chips, `24px` (`xl`) bento cards and editorial image frames, and `16px` (`lg`) nested inner previews.

```yaml
rounded:
  sm: 6px
  DEFAULT: 12px
  md: 16px
  lg: 20px
  xl: 28px
  full: 9999px
```

## Motion & Micro-interaction

Interactions feel fluid and reassuring—never abrupt. Buttons and interactive bento cards lift subtly on hover with `180ms` deceleration. Tab transitions and filter switches use clean opacity and transform easing without layout shift.

```yaml
motion:
  state-change:
    duration: 150ms
    delay: 0ms
    timingFunction: cubic-bezier(0.2, 0, 0, 1)
  card-hover:
    duration: 220ms
    delay: 0ms
    timingFunction: cubic-bezier(0.2, 0, 0, 1)
  surface-enter:
    duration: 300ms
    delay: 0ms
    timingFunction: cubic-bezier(0.16, 1, 0.3, 1)
```

## Voice & Content Strategy

- **Tone of voice:** Warm, clear, empowering, and grounded in Indonesian (`Bahasa Indonesia` yang natural, ramah, namun profesional). Avoids overly stiff medical jargon while maintaining clinical credibility.
- **Capitalisation:** Sentence case for headlines, card titles, and button labels (`Kesehatan Anda, lebih dekat dari rumah`, `Unduh aplikasi`).
- **Action labels:** Clear verb-led CTAs (`Unduh aplikasi`, `Pelajari layanan perusahaan`, `Baca artikel`).
- **Numbers & dates:** Indonesian date formatting (`26 November 2026`), transparent currency (`Rp`) with `tabular-nums`.

## Components

Component implementations live as individual HTML sticker sheet files in
`components/`. Each file contains the canonical, styled HTML components with
`<!-- COMPONENT: name -->` comment markers. Read each file before use.

- **Badge** → `@components/badge.html` — `badge-numeric`, `badge-dot`
- **Button** → `@components/button.html` — `button-primary`, `button-icon`, `button-with-icon`
- **Card** → `@components/card.html` — `card-elevated`, `card-action`, `card-interactive`, `card-interactive-2`
- **Tabs** → `@components/tabs.html` — `tabs-horizontal`, `tabs-with-icon`
