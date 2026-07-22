# File Lens — Entry-Level Interview Prep

A study sheet for talking about this project out loud. Everything here comes
from the actual code and docs. Where the code and the docs disagree, or where
intent is a guess, it says so.

> **One heads-up before you start:** the `README.md` still describes duplicate
> detection as a "size-only heuristic," but the code and `docs/DUPLICATE_DETECTION.md`
> have moved on — it now does real content-based detection (BLAKE3 hashing).
> Recent commits ("Drop the size-only duplicate guess", "Show verified
> duplicates") confirm the code is the current truth. If an interviewer quotes
> the README at you, you can calmly point out the README is stale on that one
> point. That awareness is itself a good look.

---

## 1. What is this?

**Plain explanation.** File Lens is a desktop app that helps you clean up a
messy **Downloads folder**. It looks at everything in that folder, tells you
what's taking up space and *why* (old installers, half-finished downloads,
giant files, duplicate copies), and then offers safe ways to tidy up —
sorting files into category folders or moving junk to the Recycle Bin. The
golden rule: **it never changes anything without showing you a preview first
and getting your explicit "yes."**

**Everyday analogy.** Think of a professional home organizer who comes to your
cluttered garage. They don't throw anything away on their own. They walk
through, put sticky notes on things ("this paint can is 5 years old", "you have
three identical hammers"), and hand you a written plan: "Here's what I'd move
where." You cross off what you disagree with, then say go. And nothing gets
*destroyed* — the worst that happens is a box gets moved to the curb (the
Recycle Bin), where you can still grab it back. File Lens is that organizer,
for your Downloads folder.

**Why it exists.** Downloads folders become dumping grounds: installers you
already ran, `.part` files from downloads that died, huge archives you forgot,
and many copies of the same file. Cleaning by hand is slow and risky (you might
delete something important). File Lens makes it fast *and* safe.

**Simplest usage example.** You install and open the app. It immediately shows
your last scan's results, then quietly re-scans in the background. You see a
dashboard: total files, total space used, how much you could reclaim, and a
list of flagged files each with a plain-English reason. You click into
"Organize," review the proposed plan, tweak it, and hit execute. Done — and
undoable.

---

## 2. Architecture — how it's organized

File Lens is a **Tauri** app. (Tauri = a framework for building desktop apps
where the user interface is web tech (React) and the "engine" underneath is
Rust.) The two halves talk over a typed message channel.

- **Frontend (React + TypeScript)** — draws the screens and captures what the
  user wants to do. It has **no direct access to your files.**
- **Backend (Rust)** — does all the real work: reading the disk, the database,
  the analysis, and moving files. All the "dangerous" power lives here, in one
  place.

They communicate only through a small, typed list of commands (think of it as a
fixed menu of requests the frontend is allowed to make). The whole menu lives
in one file (`src/shared/ipc/commands.ts`), so you can see the entire surface
at a glance.

### Backend modules (`src-tauri/src/`)

| Module | What it's for | What breaks if you removed it |
| ------ | ------------- | ----------------------------- |
| `filesystem` | Turns one file on disk into structured info (name, size, type, dates). | Nothing else can learn anything about a file. |
| `scanning` | Walks the whole folder tree and builds the inventory; handles progress + cancel. | No inventory — the app has nothing to analyze. |
| `database` | Saves everything to a local SQLite database (inventory, history, settings). | App forgets everything on close; every launch needs a full rescan. |
| `analysis` | Read-only "rule engine" that flags files (large, old, installer, temp) and computes the summary totals. | No recommendations, no dashboard numbers. |
| `dedup` | Finds true duplicates by comparing file *contents*, not just names. | Duplicate detection disappears. |
| `cleanup` | User actions: trash a file, reveal it, ignore a suggestion, show details. | You can see problems but can't act on individual files. |
| `organization` | Sort files into category folders: classify → plan → resolve conflicts → execute → undo. | No "Smart Organization" feature at all. |
| `settings` | Loads/saves user config (folder, thresholds, ignores, theme) and applies it. | Everything runs on fixed defaults; no customization. |

### Frontend layout (`src/`)

Organized **feature-first** — grouped by what it does for the user, not by tech
type. Each feature (`scan`, `analysis`, `dashboard`, `cleanup`, `duplicates`,
`organization`, `settings`) owns its own screens and logic. Shared building
blocks (types, the command layer, formatting helpers, the logger) live under
`shared/`.

| Part | What it's for | What breaks if removed |
| ---- | ------------- | ---------------------- |
| `features/` | The actual screens and their logic. | No UI for that feature. |
| `shared/ipc/` | The only doorway to the backend. | Frontend can't talk to the backend at all. |
| `shared/types/` | Shared data shapes matching the Rust side. | Frontend and backend stop agreeing on data. |
| `components/` | Reusable UI pieces (dialogs, spinner). | Duplicated UI code everywhere. |

**The one rule to remember:** the frontend never touches the filesystem
directly — it always goes through the backend command menu. That's the whole
safety design in one sentence.

---

## 3. The core idea / main algorithm

There are two clever things worth being able to explain.

### (a) Duplicate detection as a "funnel" (the staged pipeline)

The problem: to know two files are *truly* identical, you'd normally have to
read both files completely and compare — expensive if you have thousands of
files.

The trick: **don't do the expensive check until you've cheaply ruled out
everything you can.** Files pass through a funnel of increasingly expensive
tests, and most fall out early:

1. **Same size?** (Free — sizes were already collected during the scan.) Two
   identical files *must* be the same size, so any file with a one-of-a-kind
   size can't have a twin. Most files get eliminated here for zero cost.
2. **Same "fingerprint"?** For the survivors, read just three small 64 KB
   samples (start, middle, end) and combine them. If those don't match, the
   files definitely differ — skip the full read.
3. **Same full hash?** Only for files that *still* match, read the whole file
   and compute a **BLAKE3 hash** — a short "fingerprint of the entire
   contents." Same hash = genuinely identical. Only these are ever shown to you
   as duplicates.

**Analogy.** You're finding identical books in a huge library. First you sort by
book thickness (fast, eliminates most). For same-thickness books, you glance at
the first, middle, and last page. Only for books that still look the same do you
read them cover to cover. You never read a whole book unless you truly have to.

Two supporting details worth knowing:
- **Streaming.** Full hashing reads the file in small fixed chunks, so even a
  10 GB file uses the same tiny amount of memory as a small one. (Analogy:
  pouring a lake through a garden hose bucket-by-bucket instead of trying to
  hold the whole lake.)
- **Caching.** Computed hashes are saved in the database. A saved hash is reused
  only if the file's size *and* last-modified time both still match — any edit
  changes one of those, so a stale hash can never sneak through. So a re-scan of
  unchanged files does almost no work.

### (b) The "decide, then do" split (see section 4 — it's the headline)

---

## 4. The single most impressive design decision (lead with this)

**Separate "figuring out what to do" from "actually doing it."**

All the decision-making code — scanning, classifying files, building an
organization plan, resolving naming conflicts — is **pure**: it just takes
information in and returns a decision out, without ever touching the disk.
The code that *performs* the action (moving files, deleting) is kept in its own
small, separate place.

**Why this is the thing to lead with — it buys three big things at once:**

1. **Safety.** Nothing on disk changes until you approve a plan. The preview
   you see is exactly what will happen — "the preview is the contract." The most
   destructive thing the app can do is move a file to the Recycle Bin, and only
   on your command.
2. **Testability.** Because the decision logic never touches real files, it can
   be tested with fake inputs — no real filesystem needed. That's why the
   classifier, planner, and conflict resolver all have thorough unit tests.
3. **Trust & flexibility.** You review every move and its reason before
   committing, and every executed batch can be **undone**.

**Analogy.** It's the difference between an architect and a construction crew.
The architect draws the blueprint (cheap, safe, easy to revise, nothing's been
built yet). Only when you approve does the crew build it. File Lens keeps the
architect and the crew as separate people — you always get to see and sign off
on the blueprint first.

---

## 5. Failure modes — what happens when things go wrong

The recurring theme: **one problem never blows up the whole job.**

| Situation | What the app does |
| --------- | ----------------- |
| A file can't be read (locked, permission denied, deleted mid-scan) | Logs it, skips it, keeps going. The scan/dedup finishes; the bad file is just recorded in an error list. |
| The database is broken or unusable at startup | Tries to recreate it; if that fails, falls back to an in-memory database so the app still opens. |
| A naming conflict during organization (target name already taken) | Defaults to **Keep Both** (adds a number like `a (1).txt`) — it never silently overwrites unless you explicitly choose Replace. |
| Undo, but a new file has reappeared at the original spot | Refuses to overwrite it; records the failure and moves on. Your new file is safe. |
| A file move is requested that points outside the Downloads folder (or uses `..` to try to escape) | Rejected outright. This guards against a tampered request sneaking through the frontend/backend boundary. |
| Moving a file across different drives (a plain rename fails) | Falls back to copy-then-delete automatically. |
| The scan hits an absurd number of files (probably the wrong, huge folder) | Stops at a limit, flags it, and refuses to save that scan. |
| A scan is running and you cancel | The walk stops cleanly; a cancelled scan is treated as partial and won't delete files from the saved inventory. |

There isn't really an "external dependency is down" scenario — File Lens is a
local, offline app with no server or network calls. Its "dependency" is your
own filesystem, and the answer above (skip the bad file, keep going) is how it
handles that being flaky.

---

## 6. Likely interview questions & short answers

**Q: What does File Lens actually do?**
It scans your Downloads folder, explains what's using space and why, and lets
you safely organize files into folders or move junk to the Recycle Bin — always
with a preview and your confirmation first.

**Q: Why Rust for the backend and React for the frontend?**
Rust handles all the file and database work safely and is fast and strongly
typed; React builds the UI. Tauri glues them together and keeps the UI
sandboxed away from direct file access.

**Q: How does it avoid accidentally deleting my files?**
It never permanently deletes — "delete" means move to the Recycle Bin, which is
recoverable. Organization is preview-first and undoable, and every destructive
action needs explicit confirmation.

**Q: How does duplicate detection work?**
A funnel of cheap-to-expensive checks: group by size, then compare small
sampled fingerprints, then only fully hash the survivors with BLAKE3. Only files
with identical full contents are called duplicates.

**Q: Why not just compare file names, or just sizes?**
Same name or same size doesn't mean same contents — that would give false
positives. Content hashing is the only way to be *sure* two files are identical.

**Q: What keeps the frontend from doing something dangerous?**
The frontend can't touch the filesystem at all. It can only send a fixed,
typed list of commands to the Rust backend, which does the safety checks in one
central place.

**Q: How is the code testable without a real filesystem?**
The decision logic (classifying, planning, resolving conflicts) is pure — it
takes data in and returns decisions out, with the filesystem "injected" as a
simple function in tests. So it's tested with fake inputs, no real files needed.

**Q: Where is data stored, and why SQLite?**
In a local SQLite database in the user's app-data folder. SQLite is embedded
(compiled right into the app), needs no separate server or setup, and lets the
app remember scans between launches so it doesn't rescan every time.

**Q: What happens if the app hits a file it can't read?**
It logs and skips it and keeps going — a single unreadable file never aborts a
scan or a batch. Errors are collected and reported, not fatal.

**Q: What would you improve or add next?**
(From the roadmap) A continuous-integration setup to run all tests
automatically, progress reporting for very large organization batches, and
better hidden-file detection on Windows. I'd also fix the stale README section
that still calls duplicate detection "size-based."

---

## 7. Glossary — terms you should be able to define

- **Tauri** — A framework for building desktop apps with a web-tech UI (here,
  React) and a Rust backend, communicating over a typed message channel.
- **Frontend / Backend** — Frontend = the visible UI. Backend = the behind-the-
  scenes engine that does the real work (files, database, analysis).
- **IPC (Inter-Process Communication)** — How the frontend and backend talk. In
  Tauri, the frontend calls named "commands" and gets typed results back.
- **Command (in this app)** — One allowed request the frontend can send the
  backend (e.g. `scan_downloads`, `trash_file`). The full list lives in one file.
- **SQLite** — A small, self-contained database that lives in a single file on
  disk, with no separate server. Used here to remember scans, history, settings.
- **Inventory** — The list of all files found by a scan, with their metadata.
- **Metadata** — Facts *about* a file (name, size, type, dates) — not its
  contents.
- **Rule engine** — The analysis part: a set of simple rules (large? old?
  installer? temp?) each of which flags matching files with a plain-English
  reason.
- **Finding** — One flagged file plus the reason it was flagged.
- **Reclaimable space** — How much disk you could free, counting each file once
  and keeping one copy of each duplicate group.
- **Hash** — A short string computed from a file's entire contents; identical
  contents always produce the same hash, so it's used to prove files are equal.
- **BLAKE3** — The specific, fast hashing algorithm used here. (MD5/SHA-1 were
  deliberately avoided because they're not reliably collision-resistant.)
- **Fingerprint (sampled)** — A cheap partial signature from three 64 KB samples
  of a file, used to quickly rule files *out* before full hashing. Not proof of
  equality on its own.
- **Streaming (a file)** — Reading a file in small chunks so memory use stays
  constant no matter how big the file is.
- **Cache (hash cache)** — Saved hashes reused only when a file's size and
  modified-time still match, so unchanged files aren't re-hashed.
- **Classification** — Deciding a file's category (Documents, Images, Videos,
  Audio, Archives, Installers, Code, Other) by its extension, then its MIME type.
- **MIME type** — A standard label for a file's kind (e.g. `image/png`), used as
  a fallback when the extension is unknown.
- **Pure function** — Code that only turns inputs into outputs with no side
  effects (doesn't touch disk, network, etc.). Easy to test and reason about.
- **Side effect** — An action that changes the outside world (moving/deleting a
  file, writing to the database).
- **Plan / Preview-first** — The organizer proposes a full plan you review and
  edit; nothing moves until you execute it.
- **Conflict strategy** — What to do when a destination name is taken: Skip,
  Rename, Replace, or Keep Both (the safe default — adds a number).
- **Undo** — Every executed organization batch is recorded so its moves can be
  reversed later.
- **Trash / Recycle Bin** — "Delete" here means move to the OS Recycle Bin
  (recoverable), never permanent deletion.
- **Path validation / directory traversal** — Checking that a file path stays
  inside the allowed folder and doesn't use `..` to escape — a safety check at
  the boundary between frontend and backend.
- **Symlink (symbolic link)** — A shortcut that points at another file. The
  scanner deliberately does *not* follow these (avoids loops and surprises).
- **Upsert** — Insert-or-update: a rescan updates existing file rows instead of
  creating duplicates.
- **Concurrency / worker pool** — Running several hashing jobs at once on a fixed
  number of threads pulling from a shared queue, capped because disk work
  doesn't speed up with unlimited threads.
- **Vite / Vitest / Biome** — Build tool, test runner, and lint+format tool for
  the frontend, respectively.
</content>
</invoke>
