import { describe, expect, test } from "vitest";
import { constantTimeEqual, isWellFormedId, newRandomId } from "./session.ts";
import { MemoryBackend } from "./testing/memory-backend.ts";
import { sessionSuite } from "./testing/session-suite.ts";

const backends: MemoryBackend[] = [];

sessionSuite("MemoryBackend", {
  backend(clock) {
    const b = new MemoryBackend(clock.now);
    backends.push(b);
    return b;
  },
  dump(app) {
    const b = backends.at(-1);
    if (b === undefined) {
      throw new Error("backend belum dibuat");
    }
    const keys = b.keys().filter((k) => k.startsWith(`rs:bff:${app}:`));
    return Promise.resolve({ keys, text: JSON.stringify(keys.map((k) => [k, b.raw(k)])) });
  },
});

describe("ID acak dan perbandingan", () => {
  test("ID sesi 256 bit base64url, unik", () => {
    const ids = new Set(Array.from({ length: 200 }, () => newRandomId()));
    expect(ids.size).toBe(200);
    for (const id of ids) {
      expect(isWellFormedId(id)).toBe(true);
    }
  });

  test("constantTimeEqual membedakan nilai dan panjang berbeda", () => {
    expect(constantTimeEqual("abc", "abc")).toBe(true);
    expect(constantTimeEqual("abc", "abd")).toBe(false);
    expect(constantTimeEqual("abc", "abcd")).toBe(false);
  });
});
