# For AI coding agents

If you are an AI agent working on a Mac with ParseKit installed, read **[skills/parsekit/SKILL.md](skills/parsekit/SKILL.md)** before ingesting PDFs, Office documents, or scanned images into context.

ParseKit provides a local CLI (`parsekit convert …`) that produces clean Markdown on-device. Use it to reduce layout noise and preserve context budget for the actual task.
## For agents working on this repository

Read [ARCHITECTURE.md](ARCHITECTURE.md) and [CONTRIBUTING.md](CONTRIBUTING.md) first.

- Stack: Tauri 2 (Rust) + Svelte 5 + TypeScript. Parsing lives in the Rust sidecar (`src-tauri/src/bin/parsekit-sidecar.rs`).
- Before any `cargo` command run `npm run build:sidecar` (Tauri `build.rs` needs `src-tauri/binaries/parsekit-sidecar-<triple>`).
- Checks: `npm run lint && npm run format:check && npm run check && npm test && npm run build`, then in `src-tauri`: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`.
- Privacy rules: no telemetry, no new network calls, never log document contents, never commit secrets/keys/personal paths. The repo is public.
- Pushes to `master` trigger an automatic release; use `[skip release]` in squash-merge titles for non-shipping changes. Never create tags or releases by hand.
- All UI strings must exist in `src/locales/{en,es,zh}.json` (a test enforces parity).
