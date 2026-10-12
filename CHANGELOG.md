# Changelog

All notable changes are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [SemVer](https://semver.org/). Per-release user-facing notes also live in `docs/releases/`.

## Unreleased

### Added
- SECURITY.md, PRIVACY.md, ARCHITECTURE.md, CONTRIBUTING.md, and this changelog.
- ESLint and Prettier, `npm run lint` / `format:check`, and stricter CI (lint, audit, clippy, rustfmt).
- Dependabot configuration and issue/PR templates.
- Production-readiness audit in `docs/PRODUCTION_READINESS.md`.

### Security
- Cleared all `npm audit` findings (dev tooling: vite, svelte, vitest).
- CSP now also sets `object-src 'none'`, `base-uri 'none'`, and `form-action 'none'`.

## 0.2.12

Flow metrics scoreboard and first-run convert hero.

## 0.2.10

UI craft and motion polish. See [docs/releases/v0.2.10.md](docs/releases/v0.2.10.md).

## 0.2.9

See [docs/releases/v0.2.9.md](docs/releases/v0.2.9.md).
