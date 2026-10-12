# Security Policy

## Supported versions

Only the latest release on [GitHub Releases](https://github.com/harshabala/parsekit/releases/latest) receives security fixes. The in-app updater delivers them.

## Reporting a vulnerability

Please **do not open a public issue** for security problems.

Use GitHub's private reporting: **Security tab -> Report a vulnerability** on
[harshabala/parsekit](https://github.com/harshabala/parsekit/security/advisories/new).
Include the ParseKit version (Settings -> About), macOS version, and reproduction steps.
Please do not attach documents that contain real personal data.

You can expect an acknowledgement within a few days. This is a single-maintainer project, so response times are best-effort.

## Threat model (short)

ParseKit is a local, offline macOS menu-bar app that parses **untrusted documents** (PDF, Office, images).

- **Documents are untrusted input.** Parsing runs in a separate sidecar process (`parsekit-sidecar`) that communicates with the UI only through JSON lines on stdin/stdout. A crash or hang in the parser (300 s per-file timeout) cannot take down the UI.
- **No network access for document content.** The webview CSP only allows `self` and Tauri IPC (`connect-src 'self' ipc: http://ipc.localhost`). Network is used only by the updater (GitHub Releases) and optional OCR language-pack download. There is no telemetry.
- **Least-privilege webview.** Capabilities are listed in `src-tauri/capabilities/default.json`. The shell plugin may only spawn the bundled sidecar with no extra argv; the arbitrary `open` command is not exposed. File and folder access goes through Rust IPC commands that canonicalize paths and reject control characters (`validate_user_path`), not through a filesystem plugin.
- **Signed updates.** Updates are verified against the minisign public key embedded in `src-tauri/tauri.conf.json`. The private key is never committed; see [docs/RELEASING.md](docs/RELEASING.md).
- **Not notarized yet.** Releases are ad-hoc signed; Gatekeeper requires a one-time manual approval (see [docs/INSTALL.md](docs/INSTALL.md)).

## Out of scope

- Vulnerabilities in LibreOffice, ImageMagick, or Tesseract themselves (report upstream); ParseKit invokes them as optional external tools.
- Attacks that require an already-compromised macOS user account.

## Dependency hygiene

CI runs `npm audit` and `cargo audit`; Dependabot proposes updates weekly. See [docs/PRODUCTION_READINESS.md](docs/PRODUCTION_READINESS.md) for the current status.
