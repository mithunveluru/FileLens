# Contributing

Thanks for working on File Lens. This guide keeps the codebase consistent
and easy to maintain.

## Setup

```bash
pnpm install      # installs deps and registers the pre-commit git hook
pnpm tauri dev    # run the app
```

## Before you commit

The pre-commit hook runs `pnpm lint` and `pnpm typecheck` automatically. To run
the full set of checks yourself:

```bash
pnpm lint         # Biome lint + format check
pnpm typecheck    # TypeScript, no emit
pnpm test         # Vitest unit tests
cd src-tauri && cargo fmt --check && cargo clippy && cargo test
```

Run `pnpm lint:fix` to auto-apply formatting and safe lint fixes.

## Conventions

- **Architecture first.** Read [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md).
  Respect the layering: the frontend talks to Rust only through
  `src/shared/ipc/commands.ts`.
- **Feature-first.** New UI work goes under `src/features/<feature>/`. Promote
  code to `components/` or `shared/` only when a second feature needs it.
- **Single responsibility.** One job per module. Keep functions small (roughly
  ≤ 50 lines) and prefer pure functions.
- **Strict typing.** No `any`. Types crossing the IPC boundary live in
  `src/shared/types`.
- **No dumping grounds.** No `utils`, `helpers`, `common`, or `misc` folders.
  Name folders after what they do.
- **Tests for real logic.** Add a unit test when you write a non-trivial pure
  function or a branch worth protecting. Trivial code does not need a test.
- **Planning-first for destructive work.** Anything that mutates the filesystem
  proposes a reviewable plan first and only acts on explicit user approval; keep
  decision logic pure and put the side effects in one place. See
  [docs/SMART_ORGANIZATION.md](./docs/SMART_ORGANIZATION.md).
- **No dead code, commented-out code, or `TODO` placeholders** unless agreed in
  the issue/PR first.

## Commits & PRs

- Keep changes scoped to one concern.
- Make sure the app still compiles and runs (`pnpm tauri dev`) before opening a PR.
- Update `README.md` and `docs/ARCHITECTURE.md` when behavior or structure changes.
