# Releasing ParseKit

Maintainer notes for building, signing, and publishing a release. If you just want to use the app, ignore this file.

## Build outputs

After a release build on Apple Silicon:

| Artifact | Path |
|----------|------|
| App bundle | `src-tauri/target/release/bundle/macos/ParseKit.app` |
| DMG installer | `src-tauri/target/release/bundle/dmg/ParseKit_<version>_aarch64.dmg` |
| Updater tarball | `ParseKit_<version>_aarch64.app.tar.gz` (created by publish script) |
| Update manifest | `parsekit-latest.json` |

`src-tauri/binaries/` is gitignored. The sidecar must be built on the target architecture before packaging.

## Automated releases (default)

**You do not need to manually build or upload DMGs.** On every push to `master`:

1. CI runs unit tests and typechecks.
2. If there are new commits since the last release tag, the workflow auto-bumps the **patch** version (e.g. `0.2.4` → `0.2.5`).
3. It builds the signed `.app`, styled DMG, updater tarball, and `parsekit-latest.json`.
4. It publishes a GitHub Release and commits the synced version files back to `master` with `[skip release]` so the bot does not loop.

Push code → wait ~25 minutes → fresh DMG on [Releases](https://github.com/harshabala/parsekit/releases/latest).

To ship a **minor/major** bump instead of patch, set the version in `package.json` higher than the latest published release before pushing (e.g. `0.3.0`). The workflow keeps your manual bump.

To re-run without new commits: **Actions → Release → Run workflow**.

## Release notes

Before shipping, write user-facing notes at:

```
docs/releases/vX.Y.Z.md
```

`scripts/publish-release.sh` loads that file via `scripts/release-notes.sh`.  
If the file is missing, a clear install-focused default is used.  
Do **not** pass raw commit subjects as `RELEASE_NOTES` (that used to produce empty one-liners on GitHub).

Override only when needed:

```bash
RELEASE_NOTES="$(cat docs/releases/v0.2.10.md)" npm run publish:macos
```

## Manual release (optional)

```bash
# Local build only (sign, verify, DMG)
npm run release:macos

# Manual upload to GitHub (normally handled by CI)
export TAURI_SIGNING_PRIVATE_KEY="$HOME/.tauri/parsekit.key"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
npm run publish:macos
```

Build on each target platform (Apple Silicon vs Intel) so the sidecar filename embeds the correct host triple.

## Signing key (one-time setup)

```bash
npx tauri signer generate -w ~/.tauri/parsekit.key -f --ci -p ''
```

Keep the private key local. Never commit it. The public key lives in `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`.

## Updater flow

ParseKit uses the [Tauri v2 updater](https://v2.tauri.app/plugin/updater/). Clients fetch:

```
https://github.com/harshabala/parsekit/releases/latest/download/parsekit-latest.json
```

**Important:** GitHub returns 404 for `latest.json`. The manifest must be named **`parsekit-latest.json`**.

The updater tarball is built **after** `scripts/postbuild-macos.sh` seals the `.app` with `codesign --deep`. `createUpdaterArtifacts` is **off** in `tauri.conf.json` so Tauri doesn't emit a pre-sign tar.gz that fails strict verification on extract.

When a newer version is available, users see a gold banner: **Install & Restart** or **Later**. Updates replace the app bundle in place — not the DMG flow.

## DMG installer

The DMG uses a custom background (`scripts/dmg/dmg-background.png`) with baked-in drag-to-Applications guidance. Finder can't animate that window live, so the art carries the instructions.

Window layout is configured in `src-tauri/tauri.conf.json` under `bundle.macOS.dmg`.

## Ad-hoc signing

Release builds are ad-hoc signed (`codesign --verify --deep --strict` passes on the build artifact). They are **not** notarized. Users need the one-time Gatekeeper approval documented in [INSTALL.md](INSTALL.md).

## GitHub release assets

`publish:macos` uploads to the `v<version>` release:

- `ParseKit_<version>_aarch64.dmg`
- `ParseKit_<version>_aarch64.app.tar.gz`
- `parsekit-latest.json`

The repo must be **public** so clients can download assets without authentication.
## Updater signing: configuration reference

| Item | Where | Notes |
|------|-------|-------|
| Public key (minisign) | `src-tauri/tauri.conf.json` -> `plugins.updater.pubkey` | Public; safe to commit. Embedded in every shipped app |
| Private key | GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY` (CI) or `~/.tauri/parsekit.key` (local) | **Never commit.** `.gitignore` blocks `*.key` and `.tauri/` |
| Key password | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (empty today) | Consider setting a password and storing it as a secret |
| Manifest | `parsekit-latest.json` on the latest GitHub Release | Contains per-platform URL and signature |

**Key rotation caveat:** installed apps trust only the embedded public key. If the private key is lost or rotated, existing installs cannot auto-update to builds signed with a new key; ship the new public key in a release signed with the *old* key first, or ask users to reinstall from the DMG.

## Versioning and changelog

- SemVer. CI auto-bumps the **patch** version on each release push; set a higher version in `package.json` for minor/major.
- Version is mirrored in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` (`scripts/sync-version.sh`).
- Keep [CHANGELOG.md](../CHANGELOG.md) current under `## Unreleased`; on release, rename that heading to the version and add `docs/releases/vX.Y.Z.md`.

## Pre-release checklist

1. `npm run lint && npm run format:check && npm run check && npm test && npm run build`
2. `cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
3. `npm audit --omit=dev` and `cargo audit` report no actionable findings.
4. Release notes exist at `docs/releases/vX.Y.Z.md`.
5. Smoke test the built `.app` on a clean user account: onboarding, PDF convert, OCR, update check.

## Important: merging to master releases

Every push to `master` runs the Release workflow unless the head commit message contains `[skip release]`. For docs, CI, or tooling-only changes, include `[skip release]` in the squash-merge title.

## Rollback

Delete or mark the bad GitHub Release as a pre-release so `releases/latest` points at the previous good version; the updater then stops offering the bad build. Publish a fixed patch release afterwards. Do not re-use a version number.

## Notarization (not yet done)

Requires an Apple Developer ID certificate and notarization credentials, supplied as CI secrets. Until then, builds are ad-hoc signed.
