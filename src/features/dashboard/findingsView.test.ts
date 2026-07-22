import { describe, expect, it } from "vitest";
import { basename } from "@/shared/format/path";
import type { Finding } from "@/shared/types";
import { applyView, type ViewOptions } from "./findingsView";

const finding = (
  path: string,
  category: Finding["category"],
  sizeBytes: number,
  modifiedMs: number | null,
): Finding => ({
  path,
  category,
  reason: "",
  sizeBytes,
  modifiedMs,
});

const sample: Finding[] = [
  finding("/dl/big.zip", "largeFile", 3000, 30),
  finding("/dl/old.txt", "oldFile", 100, 10),
  finding("/dl/app.dmg", "installer", 2000, 20),
];

const base: ViewOptions = { search: "", category: "all", sort: "sizeDesc", page: 0, pageSize: 50 };

describe("basename", () => {
  it("returns the last path segment", () => {
    expect(basename("/dl/sub/a.txt")).toBe("a.txt");
    expect(basename("C:\\downloads\\b.exe")).toBe("b.exe");
  });
});

describe("applyView", () => {
  it("filters by category", () => {
    const { rows, total } = applyView(sample, { ...base, category: "installer" });
    expect(total).toBe(1);
    expect(rows[0].path).toBe("/dl/app.dmg");
  });

  it("searches by path substring, case-insensitively", () => {
    const { rows } = applyView(sample, { ...base, search: "BIG" });
    expect(rows.map((r) => r.path)).toEqual(["/dl/big.zip"]);
  });

  it("sorts by size descending and ascending", () => {
    expect(applyView(sample, { ...base, sort: "sizeDesc" }).rows[0].path).toBe("/dl/big.zip");
    expect(applyView(sample, { ...base, sort: "sizeAsc" }).rows[0].path).toBe("/dl/old.txt");
  });

  it("sorts oldest first by modified time", () => {
    expect(applyView(sample, { ...base, sort: "oldest" }).rows[0].path).toBe("/dl/old.txt");
  });

  it("paginates and clamps an out-of-range page", () => {
    const opts = { ...base, pageSize: 2 };
    const first = applyView(sample, opts);
    expect(first.rows).toHaveLength(2);
    expect(first.pageCount).toBe(2);

    const clamped = applyView(sample, { ...opts, page: 99 });
    expect(clamped.page).toBe(1);
    expect(clamped.rows).toHaveLength(1);
  });
});
