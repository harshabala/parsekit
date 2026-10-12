# Contributing to ParseKit

Thanks for helping. ParseKit is a small macOS app; keep changes focused and privacy-preserving (no telemetry, no new outbound network calls).

## Prerequisites

- macOS (Apple Silicon is the supported target)
- Node.js 20+ and npm
- Rust (stable) with `rustfmt` and `clippy`
- Xcode Command Line Tools; full Xcode is only needed to produce a signed DMG
- Optional runtime tools for testing conversion: LibreOffice (Office files), ImageMagick (images)

## Setup

```bash
git clone https://github.com/harshabala/parsekit.git
cd parsekit
npm ci
npm run build:sidecar   # required before any cargo build/test
npm run tauri:dev
```

`build:sidecar` compiles the Rust sidecar into `src-tauri/binaries/` (git-ignored). If a build is missing it, run this first.

## Checks (run before opening a PR)

```bash
npm run lint            # ESLint
npm run format:check    # Prettier
npm run check           # svelte-check / TypeScript
npm test                # Vitest
npm run build           # frontend bundle
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs the same set plus `npm audit` and `cargo audit`.

## Conventions

- Conventional commit messages (`feat:`, `fix:`, `docs:`, `chore:`, `test:`).
- Add or update tests for logic changes (`src/lib/*.test.ts`, Rust `#[cfg(test)]` modules).
- Every user-visible string goes in all three locale files (`src/locales/en.json`, `es.json`, `zh.json`); `npm test` enforces key parity.
- Never commit secrets, signing keys (`*.key`, `.tauri/`), `.env` files, personal paths, or real documents. Test fixtures must be synthetic.
- Do not log document contents.
- Do not bump versions by hand in a feature PR; see [docs/RELEASING.md](docs/RELEASING.md). Add user-facing notes under `## Unreleased` in [CHANGELOG.md](CHANGELOG.md).

## Merging to `master`

Pushes to `master` trigger the Release workflow. Put `[skip release]` in the squash-commit message for changes that should not ship a new version.

## Reporting security issues

See [SECURITY.md](SECURITY.md); do not file public issues for vulnerabilities.
