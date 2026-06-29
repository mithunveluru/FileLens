import { describe, expect, it } from "vitest";
import { recomputeDestination } from "./destination";

describe("recomputeDestination", () => {
  it("places the file in the chosen category folder under the root", () => {
    expect(recomputeDestination("/dl", "images", "/dl/photo.png")).toBe("/dl/Images/photo.png");
    expect(recomputeDestination("/dl", "code", "/dl/main.rs")).toBe("/dl/Code/main.rs");
  });

  it("preserves Windows separators and trims a trailing slash", () => {
    expect(recomputeDestination("C:\\dl\\", "documents", "C:\\dl\\a.pdf")).toBe(
      "C:\\dl\\Documents\\a.pdf",
    );
  });
});
