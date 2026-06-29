import { describe, expect, it } from "vitest";
import { formatBytes } from "./bytes";

describe("formatBytes", () => {
  it("keeps small values in bytes", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(512)).toBe("512 B");
  });

  it("scales to larger units with one decimal", () => {
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(1024 * 1024)).toBe("1.0 MB");
    expect(formatBytes(3 * 1024 ** 3)).toBe("3.0 GB");
  });
});
