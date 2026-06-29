import { describe, expect, it } from "vitest";
import { shouldLog } from "./logger";

describe("shouldLog", () => {
  it("emits when the level meets or exceeds the minimum", () => {
    expect(shouldLog("error", "info")).toBe(true);
    expect(shouldLog("info", "info")).toBe(true);
  });

  it("suppresses when the level is below the minimum", () => {
    expect(shouldLog("debug", "info")).toBe(false);
    expect(shouldLog("info", "warn")).toBe(false);
  });
});
