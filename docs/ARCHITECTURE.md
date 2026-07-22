# Architecture

This document is the single source of truth for how File Lens is laid out
and *why*. Update it whenever the structure or a key decision changes.

## High-level shape

File Lens is a [Tauri](https://tauri.app) app: a Rust backend that owns all
filesystem, database, and analysis work, and a React frontend that only renders
and dispatches user intent. The two communicate exclusively over Tauri's typed
IPC ("commands").

```
┌──────────────────────────┐        invoke()        ┌──────────────────────────┐
│  React + TypeScript (UI)  │ ─────────────────────▶ │   Rust backend (logic)    │
│  rendering, user intent   │ ◀───────────────────── │ fs · scan · analyze · db  │
└──────────────────────────┘     typed results      └──────────────────────────┘
```

**Why this split:** the frontend is sandboxed and should never touch the
filesystem directly. Keeping all privileged, testable business logic in Rust
gives us one place to enforce safety (e.g. "never delete automatically") and a
clean, mockable boundary in between.

## Boundary rules

- The frontend reaches the backend **only** through `src/shared/ipc/commands.ts`.
  No feature calls Tauri's `invoke` directly.
- Each Rust `#[tauri::command]` has exactly one matching function in
  `commands.ts`, so the entire IPC surface is discoverable in one file.
- Types shared across the boundary live in `src/shared/types`. A Rust struct
  serialized to the frontend has a matching TypeScript `interface` there.

## Frontend layout (`src/`)

Feature-first: code is grouped by what it does for the user, not by technical
type. Shared, cross-feature building blocks live under `shared/`.

| Path               | Responsibility                                                        |
| ------------------ | -------------------------------------------------------------------- |
| `main.tsx`         | Composition root: mounts React, loads global styles.                 |
| `App.tsx`          | Top-level app shell.                                                  |
| `features/`        | Self-contained feature modules (scan, analysis, cleanup, settings…). |
| `pages/`           | Top-level routed views that compose features.                        |
| `components/`      | Reusable, presentational UI primitives shared across features.       |
| `hooks/`           | Reusable React hooks shared across features.                         |
| `shared/types/`    | Types shared across features and the IPC boundary.                   |
| `shared/ipc/`      | Typed wrappers around Tauri commands (`client.ts`, `commands.ts`).   |
| `shared/config/`   | Typed access to build-time env config.                               |
| `shared/logging/`  | Frontend logger (forwards to the shared log sinks).                  |
| `styles/`          | Global styles and theme.                                             |

A feature folder owns its own components, hooks, and types; it only promotes
something to `components/`, `hooks/`, or `shared/` once a second feature needs it.

## Backend layout (`src-tauri/src/`)

| Path        | Responsibility                                              |
| ----------- | ---------------------------------------------------------- |
| `main.rs`     | Binary entry point; delegates to the library.            |
| `lib.rs`      | Composition root: builds the Tauri app, registers plugins, state, and commands. |
| `filesystem/` | Turns a single file on disk into a typed `FileEntry`. No directory walking. |
| `scanning/`   | Recursively walks a folder into a `ScanOutcome`; owns the scan commands and cancel state. |
| `database/`   | SQLite persistence (`rusqlite`, bundled): schema, scan/file upsert, history. |
| `analysis/`   | Read-only rule engine that flags files (large, old, installer, temp). |
| `dedup/`      | Verified duplicate detection: size grouping, sampled fingerprint, BLAKE3 hash, cache. |
| `cleanup/`    | User actions: trash (to Recycle Bin), reveal, ignore/unignore, file info. |
| `organization/` | Smart Organization: classify, plan, resolve conflicts, execute, undo. |
| `settings/`   | User configuration: load/save, threshold + ignore rules applied to scan/analysis. |

`scanning::scan` is deliberately Tauri-free — it takes a cancel flag and a
progress callback — so the walk logic is unit-tested without a running app. The
`scanning::commands` submodule wires it to IPC.

`settings::commands::active_root` is the single root resolver. Every command that
reads or writes user files goes through it, so scanning, organizing, and trashing
can never disagree about which folder is in scope.

## Persistence

SQLite via `rusqlite` (the `bundled` feature compiles SQLite in — no system
dependency). The database lives in the per-user app-data dir
(`file_lens.db`) and is opened once at startup into a `Mutex<Connection>`
held in Tauri state.

Schema (`src-tauri/src/database/schema.sql`):

| Table           | Purpose                                                       |
| --------------- | ------------------------------------------------------------ |
| `scans`         | One row per scan run — the history.                          |
| `files`         | Current inventory; `path` is `UNIQUE` so rescans **upsert**. |
| `settings`      | Key/value user settings (one JSON document under key `app`). |
| `ignored_paths` | Paths the user excluded from analysis.                      |

Rescan strategy: each scan upserts files by path (no duplicate rows) and stamps
`last_scan_id`. A **complete** scan then deletes files it did not see (they were
removed from disk); a **cancelled** scan is partial and never prunes. All of
this happens in one transaction. Indexes on `size_bytes`, `modified_ms`,
`extension`, and `last_scan_id` support the analysis and dashboard queries.

`hash_cache` stores full content hashes for duplicate detection, keyed by path.
See [DUPLICATE_DETECTION.md](./DUPLICATE_DETECTION.md).

## Analysis engine

`analysis::analyze` is read-only: it takes the inventory plus thresholds
(`AnalysisInput`) and returns `Finding`s, each carrying a human-readable
`reason` — the app always explains *why*. It never reads file contents or
modifies anything.

Rules are plain functions, `fn(&AnalysisInput) -> Vec<Finding>`, collected in a
`const RULES` slice. **Adding a rule = write the function and append it to the
array** — no traits, no registration boilerplate. Current rules: large files,
old files, installers, and temporary files.

The engine never reads file bytes, so it does not detect duplicates. That is the
`dedup` module's job, and it verifies by content hash rather than by metadata.

`analyze_downloads` returns an `AnalysisReport { summary, findings }`. The
`summary` (total files, total bytes, reclaimable bytes, per-category counts) is
computed once in the backend so there is a single tested source of truth —
reclaimable space counts a file flagged by several rules only once.

## Dashboard

The dashboard (`src/features/dashboard/`) renders the report: headline stat
cards plus a findings table with search, category filter, sort, and pagination.
All of that view logic is a pure function, `applyView(findings, options)`
(`findingsView.ts`), so it is unit-tested independently of React. For a
Downloads folder's scale (hundreds–thousands of files) this client-side
filtering is simpler and fast enough; server-side pagination is unnecessary.
The dashboard handles loading, error, empty-inventory, and no-matches states.

## Cleanup actions

The app never permanently deletes. "Trash" uses the `trash` crate to move a
file to the OS Recycle Bin (recoverable from there), and the backend refuses to
trash anything outside the Downloads folder (`is_within`, unit-tested) and drops
the file from the inventory afterwards. Trashing always requires confirmation
via an in-app dialog (a React component — no extra dependency).

Ignoring a recommendation adds the path to `ignored_paths`; analysis excludes
those paths, and the action is reversible (an inline **Undo**, or `unignore`).
Reveal-in-folder uses the existing opener plugin; file info is read from the
database for a read-only preview. Action wiring lives in `useCleanup`, which
refreshes the analysis after any inventory-changing action.

## Settings

Settings are stored as a single JSON document in the `settings` table (one row,
key `app`). `#[serde(default)]` keeps stored documents forward-compatible as new
fields are added. The backend applies them: thresholds build the
`AnalysisConfig`, ignored folders/extensions feed `Settings::is_excluded` (the
same analysis filter as per-path ignores), and the Downloads-folder override
changes the scan root. Theme and auto-scan-on-startup are applied by the
frontend.

`App` is the composition root: it owns the scan, analysis, and settings
controllers (plain hooks — no global store) and coordinates them — auto-scan on
startup, re-analyze after a scan completes or settings change, and apply the
theme. `ScanPanel` and `Dashboard` receive their controllers as props and stay
presentational.

## Startup and desktop integration

Startup logic lives in composition points, not in presentational components:

- The Rust `setup` hook opens the database (recreating it or falling back to
  in-memory if it is unusable, so the app always starts) and reconciles the OS
  autostart entry with the saved `launch_on_startup` preference.
- `App` shows the cached analysis immediately on mount, then — if
  `auto_scan_on_startup` is enabled — starts a background scan and re-runs the
  analysis when it completes. The scan command runs the walk on a blocking task
  off the async runtime, so the window stays responsive.
- Root resolution (`settings::resolve_root`) prefers the configured folder, then
  the remembered last-scan location, then the OS Downloads folder. The chosen
  root is persisted after each scan when "remember last scan location" is on.

Installer metadata and icons in `tauri.conf.json` drive the native installers and
the OS application entries; `tauri-plugin-autostart` provides login autostart.

## Polish (cross-cutting)

- **Theming:** a small set of CSS custom properties (`--bg`, `--fg`,
  `--surface`, `--border`, `--muted`, `--accent`, `--danger`) defined in
  `global.css` for light and dark (by OS preference and via the `data-theme`
  override). Component CSS references the tokens, so dark mode is consistent and
  one change re-themes everything.
- **Accessibility:** global `:focus-visible` outlines; modals share
  `useModalA11y` (focus-in + Escape, also deduping that logic); `aria-label`s on
  unlabeled controls; `role="alert"`/`aria-live` on errors and scan progress.
- **Motion:** subtle modal fade/scale animations, all disabled under
  `prefers-reduced-motion`.
- **Performance:** the dashboard's filter/sort/paginate runs in a `useMemo`
  keyed on report + controls.
- **Responsive:** overview cards collapse to one column and the findings table
  scrolls horizontally on narrow windows.
- **Loading:** a shared `Spinner` for scan progress and analysis loading.

## IPC surface

| Command          | Direction | Purpose                                        |
| ---------------- | --------- | ---------------------------------------------- |
| `app_info`       | call      | App name + version (health check).             |
| `scan_downloads` | call      | Scan the Downloads folder, persist + return the inventory.|
| `cancel_scan`    | call      | Request cancellation of the running scan.       |
| `scan_history`   | call      | Return the most recent persisted scans.         |
| `analyze_downloads` | call   | Run the analysis engine; returns findings + summary totals. |
| `find_duplicates` | call    | Run the duplicate pipeline; returns verified groups + stats. |
| `cancel_duplicate_scan` | call | Request cancellation of the running duplicate scan. |
| `trash_file`     | call      | Move a Downloads file to the OS Recycle Bin.    |
| `reveal_file`    | call      | Reveal a file in the system file manager.       |
| `ignore_path` / `unignore_path` | call | Exclude / restore a path in analysis. |
| `file_info`      | call      | Full metadata for one file (preview).           |
| `get_settings` / `save_settings` | call | Read / write user settings.        |
| `generate_organization_plan` | call | Propose an organization plan (read-only). |
| `execute_organization_plan` | call | Execute an approved plan (moves files). |
| `organization_history` / `undo_organization` | call | List sessions / reverse one. |
| `scan:progress`  | event     | Running file count, emitted during a scan.      |
| `dedup:progress` | event     | Running candidate count, emitted during hashing. |

Safety invariants enforced in the backend: symlinks are never followed,
unreadable entries are logged and skipped (never fatal), and only one scan and
one duplicate run happen at a time.

## Cross-cutting decisions

- **Lint + format: Biome.** One dependency replaces ESLint + Prettier and their
  plugins. Rust uses the built-in `rustfmt` and `clippy`.
- **State management: React built-ins for now.** No Redux/Zustand until there is
  genuinely shared client state that hooks + context cannot handle cleanly.
- **UI library: none.** Added only when a screen genuinely needs one, to avoid
  premature dependencies.
- **Env config: Vite's native `.env`.** No extra library; access is centralised
  and typed in `shared/config/env.ts`.
- **Logging: `tauri-plugin-log`.** Frontend and backend logs share the same
  sinks, so there is one place to read what the app did.
- **Path alias `@/` → `src/`.** Defined once in `tsconfig.json` and shared with
  Vite/Vitest via `vite-tsconfig-paths`.
