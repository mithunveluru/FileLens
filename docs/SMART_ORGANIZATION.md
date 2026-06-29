# Smart Organization

Smart Organization sorts loose files in the Downloads folder into category
subfolders (Documents, Images, Videos, Audio, Archives, Installers, Code,
Other). It is **planning-first**: the app proposes a complete plan, the user
reviews and edits it, and only an explicit execution touches the filesystem.
Every execution is recorded and can be undone.

## Why planning-first

Moving a user's files is high-trust, hard-to-reverse work. Separating *deciding
what to do* from *doing it* gives us:

- **Safety** — nothing changes until the user approves; the preview is the
  contract.
- **Trust** — the user sees every move, its reason, and any conflict before
  committing.
- **Testability** — the decision logic (classify, plan, resolve conflicts) is
  pure and deterministic, so it is unit-tested without a filesystem.
- **Extensibility** — custom rules, scheduling, or cloud sync can all produce or
  consume an `OrganizationPlan` without changing how execution works.

## Flow

```
Inventory (existing scan) → Classify → Build plan → Preview & edit
                                                        ↓ (user approves)
                                          Execute → Record session → Undo (optional)
```

The scanner and database are reused — organization reads the already-persisted
inventory rather than re-scanning, keeping scanning and organizing separate.

## Modules (`src-tauri/src/organization/`)

| Module        | Responsibility | Purity |
| ------------- | -------------- | ------ |
| `classifier`  | `FileEntry` → `FileKind` by extension, then MIME. The single source of classification rules. | pure |
| `planner`     | Inventory → `OrganizationPlan`. Organizes only top-level files; `exists` is injected for conflict flags. | pure (injected fs) |
| `conflict`    | Resolves a taken destination per strategy (Skip / Rename / Replace / Keep Both). | pure (injected fs) |
| `execution`   | Applies an approved plan and reverses sessions (undo). The only module that touches disk. | impure |
| `commands`    | Orchestrator: wires the above to IPC and the database. No business logic. | impure |

## Domain model

`FileKind`, `ClassificationResult`, `OrganizationAction` (source, destination,
kind, reason, conflict, strategy, status), `OrganizationPlan` (root, actions,
summary), `ExecutionResult`, `OrganizationSessionRecord`, `UndoResult`. The
existing `FileEntry` serves as the file-metadata model — it is not duplicated.

## Safety guarantees

- Execution **re-resolves the Downloads root server-side** and refuses any
  source or destination outside it, so a tampered `plan.root` cannot redirect
  moves.
- Conflicts default to **Keep Both** (numbered name) — execution never silently
  overwrites unless the user chooses Replace.
- A per-file failure (missing source, permission error, cross-filesystem move)
  is recorded and the batch continues; cross-filesystem moves fall back to
  copy + remove.
- **Undo never overwrites** a file that has reappeared at the original location.

## Persistence

Two additive tables: `organization_sessions` and `organization_moves` (each row
is enough to reverse a move). Undo executes the inverse moves and marks the
session `undone`.

## Frontend (`src/features/organization/`)

`useOrganizationPlan` holds the editable plan and runs execution; the plan is
edited client-side (skip a file, change its category, pick a conflict strategy)
until execution. `OrganizationView` (reached via the header **Organize** toggle)
renders the summary, the editable `PlanTable`, the execute button (with
confirmation), and `OrganizationHistory` (with undo). `recomputeDestination` is
a pure, tested helper for category changes.

## Extending

- **New category or rule:** edit the tables in `classifier.rs` — one place.
- **Custom user rules / scheduling / cloud sync:** produce an `OrganizationPlan`
  from a new source; execution, history, and undo are unchanged.
