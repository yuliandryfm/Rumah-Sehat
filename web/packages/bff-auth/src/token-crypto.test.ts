import { randomBytes } from "node:crypto";
import { CompactEncrypt, decodeProtectedHeader } from "jose";
import { describe, expect, test } from "vitest";
import { newRandomId } from "./session.ts";
import { randomKey } from "./testing/fixtures.ts";
import { TokenCryptoError, TokenSealer } from "./token-crypto.ts";

const payload = { access_token: "at-sangat-rahasia", refresh_token: "rt-sangat-rahasia" };

describe("TokenSealer (trace: ADR-011 butir 7, M-06)", () => {
  const k1 = { id: "k1", key: randomKey() };
  const sealer = new TokenSealer({ current: k1 });

  test("seal lalu open dengan ID sesi yang sama → isi kembali utuh", async () => {
    const sid = newRandomId();
    const jwe = await sealer.seal(sid, "tokens", payload);
    expect(await sealer.open(sid, "tokens", jwe)).toEqual(payload);
  });

  test("JWE Compact dir/A256GCM dengan kid; isi tidak terbaca dan ID sesi tidak ada di header", async () => {
    const sid = newRandomId();
    const jwe = await sealer.seal(sid, "tokens", payload);
    expect(jwe.split(".")).toHaveLength(5);
    expect(jwe).not.toContain("rahasia");
    expect(Buffer.from(jwe.split(".")[3] ?? "", "base64url").toString("latin1")).not.toContain("at-sangat");
    const header = decodeProtectedHeader(jwe);
    expect(header).toEqual({ alg: "dir", enc: "A256GCM", kid: "k1", cty: "rs-bff-tokens+json" });
    expect(JSON.stringify(header)).not.toContain(sid);
  });

  test("ID sesi lain → gagal dibuka (ciphertext dipindah ke sesi lain)", async () => {
    const jwe = await sealer.seal(newRandomId(), "tokens", payload);
    await expect(sealer.open(newRandomId(), "tokens", jwe)).rejects.toBeInstanceOf(TokenCryptoError);
  });

  test("tujuan lain (login vs tokens) → gagal dibuka", async () => {
    const sid = newRandomId();
    const jwe = await sealer.seal(sid, "login", payload);
    await expect(sealer.open(sid, "tokens", jwe)).rejects.toThrow(TokenCryptoError);
  });

  test("ciphertext atau tag diubah → gagal dibuka", async () => {
    const sid = newRandomId();
    const parts = (await sealer.seal(sid, "tokens", payload)).split(".");
    const ubah = (i: number): string => {
      const p = [...parts];
      const b = Buffer.from(p[i] ?? "", "base64url");
      b[0] = (b[0] ?? 0) ^ 1;
      p[i] = b.toString("base64url");
      return p.join(".");
    };
    await expect(sealer.open(sid, "tokens", ubah(3))).rejects.toThrow(TokenCryptoError);
    await expect(sealer.open(sid, "tokens", ubah(4))).rejects.toThrow(TokenCryptoError);
  });

  test("kunci induk lain (pembaca Valkey tanpa kunci OpenBao) → gagal dibuka", async () => {
    const sid = newRandomId();
    const jwe = await sealer.seal(sid, "tokens", payload);
    const lain = new TokenSealer({ current: { id: "k1", key: randomKey() } });
    await expect(lain.open(sid, "tokens", jwe)).rejects.toThrow(TokenCryptoError);
  });

  test("rotasi: JWE kunci lama dapat dibuka selama kunci lama menjadi previous; kid tak dikenal ditolak", async () => {
    const sid = newRandomId();
    const lama = await sealer.seal(sid, "tokens", payload);
    const k2 = { id: "k2", key: randomKey() };
    const baru = new TokenSealer({ current: k2, previous: k1 });
    expect(await baru.open(sid, "tokens", lama)).toEqual(payload);
    expect(decodeProtectedHeader(await baru.seal(sid, "tokens", payload)).kid).toBe("k2");
    const tanpaLama = new TokenSealer({ current: k2 });
    await expect(tanpaLama.open(sid, "tokens", lama)).rejects.toThrow(TokenCryptoError);
  });

  test("algoritma di luar daftar izin (A256KW) → ditolak walau kuncinya benar", async () => {
    const jwe = await new CompactEncrypt(new TextEncoder().encode(JSON.stringify(payload)))
      .setProtectedHeader({ alg: "A256KW", enc: "A256GCM", kid: "k1", cty: "rs-bff-tokens+json" })
      .encrypt(new Uint8Array(randomBytes(32)));
    await expect(sealer.open(newRandomId(), "tokens", jwe)).rejects.toThrow(TokenCryptoError);
  });

  test("ID sesi kosong → ditolak", async () => {
    await expect(sealer.seal("", "tokens", payload)).rejects.toThrow(TokenCryptoError);
    await expect(sealer.open("", "tokens", "a.b.c.d.e")).rejects.toThrow(TokenCryptoError);
  });
});
