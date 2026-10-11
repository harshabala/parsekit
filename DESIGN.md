---
name: ParseKit
description: Menu-bar document converter — private, local, native macOS craft
colors:
  ink: "#1B1713"
  ink-secondary: "#6B5E52"
  accent: "#3F3830"
  accent-hover: "#322C26"
  accent-fg: "#F9F7F2"
  surface: "#F7F4EE"
  surface-elevated: "#FFFFFF"
  border: "#B8A99A"
  link: "#5C4A3A"
  status-success: "#2F8A4A"
  status-error: "#C23B2E"
  status-warning: "#C9892A"
  dark-bg: "#1C1A17"
  dark-ink: "#F5F2EB"
  dark-accent: "#E8C99A"
typography:
  display:
    fontFamily: "Instrument Serif, Georgia, Times New Roman, serif"
    fontSize: "1.25rem"
    fontWeight: 400
    lineHeight: 1.2
    letterSpacing: "-0.015em"
  body:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Segoe UI, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "normal"
  mono:
    fontFamily: "JetBrains Mono, SF Mono, Menlo, monospace"
    fontSize: "0.8125rem"
    fontWeight: 500
    lineHeight: 1.3
    letterSpacing: "-0.01em"
  label:
    fontFamily: "-apple-system, BlinkMacSystemFont, SF Pro Text, Segoe UI, sans-serif"
    fontSize: "0.72rem"
    fontWeight: 500
    lineHeight: 1.25
    letterSpacing: "0.01em"
rounded:
  sm: "6px"
  md: "8px"
  lg: "12px"
  pill: "999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "24px"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.accent-fg}"
    rounded: "{rounded.md}"
    padding: "8px 14px"
  button-primary-hover:
    backgroundColor: "{colors.accent-hover}"
    textColor: "{colors.accent-fg}"
  button-secondary:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    padding: "8px 12px"
  chip:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    rounded: "{rounded.pill}"
    padding: "5px 10px"
  card:
    backgroundColor: "{colors.surface-elevated}"
    textColor: "{colors.ink}"
    rounded: "{rounded.lg}"
    padding: "14px"
---

## Overview

ParseKit is a **product-register** macOS menu-bar tool: design serves conversion, not marketing spectacle. Visual system is **Bone White, Pressed** — warm restrained neutrals, charcoal primary, native SF Pro body, Instrument Serif only for display (app title / about). Motion is short (120–180ms), ease-out, interruptible, reduced-motion aware. Identity lives in craft density and privacy posture, not loud color.

Canonical tokens: `src/index.css` (`:root`, `[data-theme]`, system dark). Brand narrative: `brand.md`. Product brief: `PRODUCT.md`.

## Colors

OKLCH is source of truth in CSS; hex above is sRGB approximation for tooling.

| Role | Light (OKLCH) | Use |
|------|----------------|-----|
| Ink | `oklch(0.18 0.01 50)` | Body text |
| Secondary | `oklch(0.48 0.03 50)` | Hints, labels (keep ≥4.5:1 on surface) |
| Accent | `oklch(0.32 0.03 50)` | Primary buttons, focus |
| Accent FG | `oklch(0.98 0.005 85)` | Text on primary |
| Surface glass | `oklch(0.97 0.01 85)` + blur | Popover material |
| Border | `oklch(0.72 0.02 50 / 0.95)` | Hairlines |
| Success / Error / Warning | status-* tokens | Semantic only |

Dark theme flips ink/surface and warms the accent (`oklch(0.78 0.08 75)`). Prefer **restrained** strategy: neutrals dominate; accent ≤10% of surface.

## Typography

- **UI / body:** system sans (`--font-sans`) — native macOS feel.
- **Display only:** Instrument Serif (`--font-serif`) for app name / about — not for buttons, data, or dense labels.
- **Data / counts:** JetBrains Mono (`--font-mono`), tabular-nums for token counts.
- Scale ratio ~1.15–1.2; display letter-spacing floor **≥ -0.04em** (hero metrics ~-0.03em).
- Prefer weight + size hierarchy over all-caps eyebrows on every section.

## Elevation

Mostly **tonal layering** (secondary-bg, glass) over heavy drop shadows.

- Prefer: 1px border **or** soft shadow ≤8px blur — not both as decoration.
- Popover: glass material + restrained inset highlight.
- Focus: `--focus-ring` translucent accent ring (never remove).

## Components

| Component | Notes |
|-----------|--------|
| **Primary button** | `.run-parse-btn` / accent fill; `:active scale(0.97)` |
| **Secondary button** | Transparent / light border; quieter press scale |
| **Chips** | Pill outline for optional choices (destination) |
| **Scoreboard** | Post-convert result: one hero number, platter stats, progressive disclosure for method |
| **Drop zone** | Dominant affordance for first-run convert |
| **Segmented control** | Period toggle (today / month / lifetime) |
| **Banners** | Error/notice with fly enter; reduced-motion → fade |

States required on interactive: default, hover, focus-visible, active, disabled.

## Do's and Don'ts

**Do**
- Keep conversion path obvious: drop → run → open output.
- Use local metrics language (estimate, privacy line).
- Animate transform/opacity only; honor `prefers-reduced-motion` and `prefers-reduced-transparency`.
- Align new UI to tokens in `src/index.css` before inventing hex.

**Don't**
- Side-stripe accents (`border-left` >1px on cards/callouts).
- Hero-metric SaaS clichés with gradients and confetti.
- Nested cards; identical icon+title+body grids.
- Uppercase tracked kicker on every block.
- `scale(0)` entrances; bounce/elastic easing.
- Pair thick border + wide soft shadow (ghost-card).
- Put Instrument Serif on dense UI chrome.
