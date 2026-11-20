// Setiap kode respons yang dipancarkan BFF terdaftar di docs/05-response-codes.md dengan HTTP dan deskripsi
// yang sama (coding-standards §3.5; registry §5 butir 3–4).

import { readFileSync } from "node:fs";
import { describe, expect, test } from "vitest";
import { BFF_CODES } from "./http.ts";
import { REPO_ROOT } from "./testing/containers.ts";

function registry(): Map<string, { http: number; description: string }> {
  const md = readFileSync(`${REPO_ROOT}docs/05-response-codes.md`, "utf8");
  const out = new Map<string, { http: number; description: string }>();
  for (const line of md.split("\n")) {
    const m = /^\|\s*`(\d{7})`\s*\|\s*(\d{3})\s*\|\s*([^|]+?)\s*\|/.exec(line.trim());
    if (m?.[1] !== undefined && m[2] !== undefined && m[3] !== undefined) {
      out.set(m[1], { http: Number(m[2]), description: m[3] });
    }
  }
  return out;
}

describe("kode respons BFF", () => {
  const reg = registry();

  test.each(Object.entries(BFF_CODES))("%s terdaftar dengan HTTP dan deskripsi sama", (_n, c) => {
    expect(reg.get(c.code)).toEqual({ http: c.http, description: c.description });
    expect(c.code.slice(0, 3)).toBe(String(c.http));
  });
});
