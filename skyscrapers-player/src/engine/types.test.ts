import { describe, expect, it } from "vitest";
import { normalizeDifficultyParam } from "./types";

describe("normalizeDifficultyParam", () => {
  it("returns the canonical value for current labels", () => {
    expect(normalizeDifficultyParam("easy")).toBe("easy");
    expect(normalizeDifficultyParam("medium")).toBe("medium");
    expect(normalizeDifficultyParam("hard")).toBe("hard");
    expect(normalizeDifficultyParam("expert")).toBe("expert");
    expect(normalizeDifficultyParam("master")).toBe("master");
  });

  it("matches case-insensitively", () => {
    expect(normalizeDifficultyParam("EXPERT")).toBe("expert");
    expect(normalizeDifficultyParam("Master")).toBe("master");
  });

  it("returns undefined for missing or invalid input", () => {
    expect(normalizeDifficultyParam(null)).toBeUndefined();
    expect(normalizeDifficultyParam(undefined)).toBeUndefined();
    expect(normalizeDifficultyParam("")).toBeUndefined();
    expect(normalizeDifficultyParam("trivial")).toBeUndefined();
    expect(normalizeDifficultyParam("grandmaster")).toBeUndefined();
  });
});
