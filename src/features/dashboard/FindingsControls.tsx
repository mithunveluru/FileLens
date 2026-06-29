import {
  CATEGORY_LABELS,
  type CategoryFilter,
  type SortKey,
  type ViewOptions,
} from "@/features/dashboard/findingsView";
import type { CategorySummary, FindingCategory } from "@/shared/types";

interface FindingsControlsProps {
  options: ViewOptions;
  categories: CategorySummary[];
  onChange: (patch: Partial<ViewOptions>) => void;
}

const SORT_LABELS: Record<SortKey, string> = {
  sizeDesc: "Largest first",
  sizeAsc: "Smallest first",
  name: "Name (A–Z)",
  oldest: "Oldest first",
};

/** Search box, category filter, and sort selector for the findings table. */
function FindingsControls({ options, categories, onChange }: FindingsControlsProps) {
  return (
    <div className="findings-controls">
      <input
        type="search"
        aria-label="Search findings by name or path"
        placeholder="Search by name or path…"
        value={options.search}
        onChange={(e) => onChange({ search: e.currentTarget.value })}
      />

      <select
        aria-label="Filter by category"
        value={options.category}
        onChange={(e) => onChange({ category: e.currentTarget.value as CategoryFilter })}
      >
        <option value="all">All categories</option>
        {categories.map((summary) => (
          <option key={summary.category} value={summary.category}>
            {CATEGORY_LABELS[summary.category as FindingCategory]} ({summary.count})
          </option>
        ))}
      </select>

      <select
        aria-label="Sort findings"
        value={options.sort}
        onChange={(e) => onChange({ sort: e.currentTarget.value as SortKey })}
      >
        {(Object.keys(SORT_LABELS) as SortKey[]).map((key) => (
          <option key={key} value={key}>
            {SORT_LABELS[key]}
          </option>
        ))}
      </select>
    </div>
  );
}

export default FindingsControls;
