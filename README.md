# File Lens

> Understand. Organize. Reclaim.

File Lens scans your Downloads folder and gives **intelligent cleanup
recommendations** — always explaining *why* — while leaving you in full control.
It never permanently deletes anything automatically; the most it will do, on your
command, is move a file to the system Recycle Bin.

## Tech stack

| Layer            | Choice                          |
| ---------------- | ------------------------------- |
| Desktop shell    | [Tauri 2](https://tauri.app)    |
| UI               | React 19 + TypeScript + Vite    |
| Backend          | Rust                            |
| Storage          | SQLite (from Phase 2)           |
| Lint + format    | [Biome](https://biomejs.dev)    |
| Tests            | Vitest (frontend) + `cargo test`|

## Prerequisites

- Node.js 20+ and [pnpm](https://pnpm.io) 9+
- Rust (stable) via [rustup](https://rustup.rs)
- Tauri's system dependencies for your OS — see the
  [Tauri prerequisites guide](https://tauri.app/start/prerequisites/).

## Getting started

```bash
pnpm install        # install JS deps and register git hooks
pnpm tauri dev      # run the desktop app (compiles Rust on first run)
```

## Common scripts

| Command            | What it does                              |
| ------------------ | ----------------------------------------- |
| `pnpm tauri dev`   | Run the full desktop app in dev mode      |
| `pnpm dev`         | Run the Vite frontend only (browser)      |
| `pnpm build`       | Type-check and build the frontend bundle  |
| `pnpm tauri build` | Build a distributable desktop binary      |
| `pnpm lint`        | Lint + format check (Biome)               |
| `pnpm lint:fix`    | Apply safe lint/format fixes              |
| `pnpm typecheck`   | Type-check without emitting               |
| `pnpm test`        | Run frontend unit tests (Vitest)          |

Rust checks: `cargo fmt`, `cargo clippy`, and `cargo test` inside `src-tauri/`.

## Documentation

- [Architecture](./docs/ARCHITECTURE.md) — folder map and design decisions.
- [Contributing](./CONTRIBUTING.md) — workflow and conventions.

## Project status

Built in phases.

- **Phase 0 — Foundation** ✅ Compiles, runs, logs, typed IPC bridge.
- **Phase 1 — File Scanner** ✅ Recursively scans the Downloads folder for file
  metadata, with live progress and cancellation. Symlinks are skipped and
  unreadable files never crash the scan.
- **Phase 2 — Database** ✅ Scans persist to SQLite (upsert by path, no
  duplicates, transactional), with scan history surfaced in the UI.
- **Phase 3 — Analysis Engine** ✅ Read-only rules flag large, old, installer,
  temporary, and duplicate files, each with a plain-language reason.
- **Phase 4 — Dashboard** ✅ Headline stats (files, disk usage, reclaimable
  space) and a searchable, filterable, sortable, paginated findings table.
- **Phase 5 — Cleanup** ✅ Move files to the Recycle Bin (with confirmation),
  ignore recommendations (with undo), reveal in the file manager, and preview
  file info. Nothing is ever permanently deleted.
- **Phase 6 — Settings** ✅ Configure the Downloads folder, age and large-file
  thresholds, ignored folders/extensions, theme, and auto-scan on startup —
  persisted to SQLite.
- **Phase 7 — Polish** ✅ Token-based theming with consistent dark mode,
  accessibility (focus, keyboard, ARIA), subtle animations (reduced-motion
  aware), responsive layouts, and loading spinners.

All planned phases are complete.

## Smart Organization

Beyond cleanup, File Lens can **organize** your Downloads into category
folders (Documents, Images, Videos, …) using a **planning-first** flow: it
proposes a full plan you review and edit, and nothing moves until you execute.
Every session is recorded and can be undone. Switch between **Clean up** and
**Organize** from the header. See
[docs/SMART_ORGANIZATION.md](./docs/SMART_ORGANIZATION.md).
