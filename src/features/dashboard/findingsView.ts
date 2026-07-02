import { basename } from "@/shared/format/path";
import type { Finding, FindingCategory } from "@/shared/types";

export { basename };

export type SortKey = "sizeDesc" | "sizeAsc" | "name" | "oldest";
export type CategoryFilter = FindingCategory | "all";

export interface ViewOptions {
  search: string;
  category: CategoryFilter;
  sort: SortKey;
  /** Zero-based page index. */
  page: number;
  pageSize: number;
}

export interface ViewResult {
  rows: Finding[];
  /** Matches after filtering, before pagination. */
  total: number;
  pageCount: number;
  /** Clamped into range. */
  page: number;
}

export const CATEGORY_LABELS: Record<FindingCategory, string> = {
  largeFile: "Large files",
  oldFile: "Old files",
  installer: "Installers",
  temporaryFile: "Temporary files",
};

export function applyView(findings: Finding[], opts: ViewOptions): ViewResult {
  const search = opts.search.trim().toLowerCase();
  const filtered = findings.filter((finding) => {
    if (opts.category !== "all" && finding.category !== opts.category) return false;
    if (search && !finding.path.toLowerCase().includes(search)) return false;
    return true;
  });

  const sorted = sortFindings(filtered, opts.sort);

  const total = sorted.length;
  const pageCount = Math.max(1, Math.ceil(total / opts.pageSize));
  const page = Math.min(Math.max(0, opts.page), pageCount - 1);
  const start = page * opts.pageSize;
  return { rows: sorted.slice(start, start + opts.pageSize), total, pageCount, page };
}

function sortFindings(findings: Finding[], sort: SortKey): Finding[] {
  const sorted = [...findings];
  switch (sort) {
    case "sizeDesc":
      return sorted.sort((a, b) => b.sizeBytes - a.sizeBytes);
    case "sizeAsc":
      return sorted.sort((a, b) => a.sizeBytes - b.sizeBytes);
    case "name":
      return sorted.sort((a, b) => basename(a.path).localeCompare(basename(b.path)));
    case "oldest":
      return sorted.sort(
        (a, b) =>
          (a.modifiedMs ?? Number.POSITIVE_INFINITY) - (b.modifiedMs ?? Number.POSITIVE_INFINITY),
      );
  }
}
