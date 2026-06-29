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

## Commands

```bash
# Local release (sign, verify, DMG)
npm run release:macos

# Release + upload to GitHub
export TAURI_SIGNING_PRIVATE_KEY="$HOME/.tauri/parsekit.key"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
RELEASE_NOTES="ParseKit v0.2.x — …" npm run publish:macos
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