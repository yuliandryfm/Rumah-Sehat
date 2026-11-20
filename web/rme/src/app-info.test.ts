import { describe, expect, test } from "vitest";
import { APP_NAME } from "./app-info.ts";

describe("kerangka rme", () => {
  test("nama aplikasi sama dengan nama folder → konsisten", () => {
    expect(APP_NAME).toBe("rme");
  });
});
