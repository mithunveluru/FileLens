const UNITS = ["B", "KB", "MB", "GB", "TB"] as const;

/** Formats a byte count as a human-readable size, e.g. `1536` -> `"1.5 KB"`. */
export function formatBytes(bytes: number): string {
  let size = bytes;
  let unit = 0;
  while (size >= 1024 && unit < UNITS.length - 1) {
    size /= 1024;
    unit += 1;
  }
  return unit === 0 ? `${bytes} ${UNITS[0]}` : `${size.toFixed(1)} ${UNITS[unit]}`;
}
