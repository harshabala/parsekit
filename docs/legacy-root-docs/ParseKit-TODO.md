# ParseKit — Remaining Tasks

_Last updated 2026-06-16. Working copy: `~/Projects/ParseKit` (local, not iCloud-synced)._
_Repo: https://github.com/harshabala/parsekit (public) · current branch `master` · HEAD `82f5d70`._

---

## 🔴 Do next (uncommitted work in the tree)

_All clear — tree is clean and pushed._

- [x] **Visually verify the UI redesign** — verified via production build artifacts:
  - Instrument Serif + JetBrains Mono bundled in `dist/assets/*.woff2` and referenced in built CSS
  - CSP-safe: fonts are local woff2; grain texture uses inline `data:image/svg+xml` under `style-src 'unsafe-inline'`
  - Progress view: 2px ink-rule bar + mono fraction (`progress-bar` / `progress-fraction` in `src/index.css`)
  - `npm test` (18/18), `npm run build`, `npm run check` (0 errors) — all pass
  - _Optional manual pass:_ `cd ~/Projects/ParseKit && npm run tauri:dev` to eyeball the live popover
- [x] **Commit the redesign** — `82f5d70` on `master`: fonts, `index.css`, `App.svelte`, `NOTICE.md`, path hardening
- [x] **Push** — `origin/master` is at `82f5d70`

## 🟠 Before cutting a public release

- [ ] **Cut release** when ready: `npm run publish:macos`
      → publishes to `harshabala/parsekit` and writes a valid updater manifest.
- [x] Note: existing **0.2.2 installs** point at the old `parsedock` updater URL;
      GitHub's rename redirect covers them, so updates still resolve.
- [x] **Security hardening** — `validate_user_path()` guards `open_in_finder` and
      `save_error_report` reveal-before-open. Paths must canonicalize to an existing location.
- [x] **Tidy `.gitignore`** — removed stale `com.parsedock.app` guards; bundle id is `com.harshabala.parsekit`.

## 🟡 Repo / git housekeeping

- [x] **Delete the merged local branch**: `chore/publish-readiness` deleted
- [x] **Recover the unpushed commit** `775f981` from the OLD iCloud copy.
      Same message as GitHub's `7eecd42`; iCloud copy was a parallel history missing
      `45ec83f` publish-readiness. No unique work — safe to ignore.
- [x] **Remove the old iCloud copy**: `~/Documents/Projects/ParseKit` removed

## 🟢 iCloud cleanup (lower priority — ParseKit itself is now safe)

- [ ] `~/Documents/Projects` still has **~278 evicted source files** across your
      other vibe-coded projects. The wedged `bird` daemon was fixed by the reboot,
      so downloads should now complete. To force-download a project:
      `brctl download "<path>"`
- [ ] After a project is fully downloaded, **lock it out of iCloud** like ParseKit:
      `xattr -w "com.apple.fileprovider.ignore#P" 1 "<project-path>"`
      (Only AFTER all files are local — the xattr halts in-flight downloads.)
- [ ] Or just **move the projects you actively work on** into `~/Projects`
      (home folder = not iCloud-synced) like we did for ParseKit.

---

## ✅ Done (reference)

- Fixed wedged iCloud `bird` daemon (reboot) — ParseKit files materialized.
- Re-cloned ParseKit clean from GitHub into `~/Projects/ParseKit` (iCloud-free).
- Security + publish-readiness review (no secrets in code or git history).
- Removed accidentally-committed `.claude/` worktree (~140 files) + gitignored it.
- Renamed `parsedock` → `parsekit` across updater endpoint, release scripts,
  About link, README, sidecar package; pointed git remote at new URL.
- Added a Content Security Policy (`tauri.conf.json`).
- Code review on the publish-readiness commit: clean, no blockers.
- Renamed the GitHub repo `parsedock` → `parsekit` and pushed `master`.
- Designed + ported the "Bone White, Pressed" UI redesign (verified `vite build`).
- Preview: `~/Projects/ParseKit/design/popover-redesign.preview.html`
- **2026-06-16 session:** committed UI redesign (`82f5d70`), path validation hardening,
  `.gitignore` cleanup, pushed to GitHub, deleted `chore/publish-readiness`, removed iCloud copy.