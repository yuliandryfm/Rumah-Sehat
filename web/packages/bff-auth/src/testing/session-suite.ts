// Suite perilaku sesi yang dijalankan dua kali: atas MemoryBackend (test unit) dan atas Valkey sungguhan
// (test integrasi), sehingga kesesuaian perilaku keduanya ikut terbukti. KHUSUS TEST.

import { describe, expect, test } from "vitest";
import { SESSION_POLICY, SessionManager, type TokenSet } from "../session.ts";
import type { SessionBackend } from "../store.ts";
import { TokenSealer } from "../token-crypto.ts";
import { TestClock, randomKey } from "./fixtures.ts";

export interface SessionSuiteHarness {
  /** Backend baru (kosong untuk awalan aplikasi ini) dengan jam test. */
  backend(clock: TestClock): SessionBackend;
  /** Seluruh isi penyimpanan mentah (kunci + nilai) untuk awalan aplikasi, sebagai teks. */
  dump(appName: string): Promise<{ keys: string[]; text: string }>;
}

const MENIT = 60 * 1000;

function tokenSet(n = 1): TokenSet {
  return {
    access_token: `at-rahasia-${String(n)}-${Math.random().toString(36).slice(2)}`,
    refresh_token: `rt-rahasia-${String(n)}-${Math.random().toString(36).slice(2)}`,
    id_token: `idt-rahasia-${String(n)}`,
    access_expires_at: 1_900_000_000_000,
    sub: "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  };
}

export function sessionSuite(nama: string, harness: SessionSuiteHarness): void {
  describe(`SessionManager atas ${nama} (trace: ADR-011 butir 7, NFR-SEC-04)`, () => {
    let seq = 0;
    const setup = (): { clock: TestClock; sm: SessionManager; app: string } => {
      seq++;
      const app = `uji${String(seq)}${Math.random().toString(36).slice(2, 8)}`;
      const clock = new TestClock();
      const sm = new SessionManager({
        appName: app,
        backend: harness.backend(clock),
        sealer: new TokenSealer({ current: { id: "k1", key: randomKey() } }),
        now: clock.now,
      });
      return { clock, sm, app };
    };

    test("sesi baru: kunci penyimpanan berupa hash, ID sesi mentah dan token tidak tersimpan apa adanya", async () => {
      const { sm, app } = setup();
      const t = tokenSet();
      const s = await sm.create(t);
      const { keys, text } = await harness.dump(app);
      expect(keys).toEqual([sm.key("s", s.id)]);
      expect(keys[0]).toMatch(new RegExp(`^rs:bff:${app}:s:[0-9a-f]{64}$`));
      expect(text).not.toContain(s.id);
      expect(text).not.toContain(t.access_token);
      expect(text).not.toContain(t.refresh_token);
      expect(text).not.toContain(t.id_token);
      expect(text).not.toContain(t.sub);
      expect(await sm.tokens(s)).toEqual(t);
    });

    test("tidak aktif 29 menit 59 detik → aktif; 30 menit → kedaluwarsa (idle) dan terhapus", async () => {
      const { clock, sm, app } = setup();
      const s = await sm.create(tokenSet());
      clock.advance(30 * MENIT - 1000);
      expect((await sm.load(s.id)).status).toBe("active");
      clock.advance(1000);
      expect(await sm.load(s.id)).toEqual({ status: "expired", reason: "idle" });
      expect((await harness.dump(app)).keys).toEqual([]);
      expect(await sm.load(s.id)).toEqual({ status: "absent" });
    });

    test("aktivitas memperpanjang batas tidak aktif", async () => {
      const { clock, sm } = setup();
      const s = await sm.create(tokenSet());
      clock.advance(20 * MENIT);
      const t = await sm.touch(s);
      expect(t?.lastActiveAt).toBe(clock.now());
      clock.advance(20 * MENIT);
      const r = await sm.load(s.id);
      expect(r.status).toBe("active");
      if (r.status === "active") {
        expect(sm.idleExpiresAt(r.session)).toBe(clock.now() + 10 * MENIT);
      }
    });

    test("batas mutlak 12 jam berlaku walau terus aktif", async () => {
      const { clock, sm } = setup();
      const s = await sm.create(tokenSet());
      expect(s.absoluteExpiresAt - s.createdAt).toBe(SESSION_POLICY.absoluteMs);
      for (let i = 0; i < 12 * 4 - 1; i++) {
        clock.advance(15 * MENIT);
        const r = await sm.load(s.id);
        expect(r.status).toBe("active");
        if (r.status === "active") {
          await sm.touch(r.session);
        }
      }
      const akhir = await sm.load(s.id);
      expect(akhir.status).toBe("active");
      if (akhir.status === "active") {
        expect(sm.idleExpiresAt(akhir.session)).toBe(s.absoluteExpiresAt);
      }
      clock.advance(15 * MENIT);
      expect(await sm.load(s.id)).toEqual({ status: "expired", reason: "absolute" });
    });

    test("sesi diakhiri: muat → absent; touch dan replaceTokens tidak menghidupkannya kembali", async () => {
      const { sm, app } = setup();
      const s = await sm.create(tokenSet());
      await sm.destroy(s.id);
      expect(await sm.load(s.id)).toEqual({ status: "absent" });
      expect(await sm.touch(s)).toBeNull();
      expect(await sm.replaceTokens(s, tokenSet(2))).toBeNull();
      expect((await harness.dump(app)).keys).toEqual([]);
    });

    test("ganti token tidak menimpa aktivitas, dan touch tidak menimpa token", async () => {
      const { clock, sm } = setup();
      const s = await sm.create(tokenSet(1));
      clock.advance(5 * MENIT);
      const touched = await sm.touch(s);
      const baru = tokenSet(2);
      // Penyegar memegang salinan sesi lama (sebelum touch): aktivitas tetap milik touch.
      expect(await sm.replaceTokens(s, baru)).not.toBeNull();
      const r = await sm.load(s.id);
      expect(r.status).toBe("active");
      if (r.status === "active") {
        expect(r.session.lastActiveAt).toBe(touched?.lastActiveAt);
        expect(await sm.tokens(r.session)).toEqual(baru);
      }
    });

    test("ID sesi tidak berbentuk sah atau tidak dikenal → absent", async () => {
      const { sm } = setup();
      for (const id of [undefined, "", "pendek", "a".repeat(44), "../".repeat(15)]) {
        expect(await sm.load(id)).toEqual({ status: "absent" });
      }
      expect(await sm.load("A".repeat(43))).toEqual({ status: "absent" });
    });

    test("transaksi login: sekali pakai, terenkripsi, kedaluwarsa 10 menit", async () => {
      const { clock, sm, app } = setup();
      const tx = { state: "st-1", nonce: "no-1", code_verifier: "cv-rahasia-1", return_to: "/antrean" };
      const id = await sm.saveLogin(tx);
      expect((await harness.dump(app)).text).not.toContain("cv-rahasia-1");
      expect(await sm.takeLogin(id)).toEqual(tx);
      expect(await sm.takeLogin(id)).toBeNull();
      const id2 = await sm.saveLogin(tx);
      clock.advance(10 * MENIT);
      // Memori: TTL mengikuti jam test. Valkey: TTL waktu nyata, jadi transaksi lama masih ada di sini;
      // karena itu batas 10 menit Valkey diuji terpisah dengan TTL nyata di test integrasi.
      if (nama === "MemoryBackend") {
        expect(await sm.takeLogin(id2)).toBeNull();
      }
      expect(await sm.takeLogin("A".repeat(43))).toBeNull();
    });

    test("kunci penyegaran: hanya satu pemegang; dilepas hanya oleh pemiliknya", async () => {
      const { sm } = setup();
      const s = await sm.create(tokenSet());
      expect(await sm.acquireRefreshLock(s, "a", 10_000)).toBe(true);
      expect(await sm.acquireRefreshLock(s, "b", 10_000)).toBe(false);
      await sm.releaseRefreshLock(s, "b");
      expect(await sm.refreshLockHeld(s)).toBe(true);
      await sm.releaseRefreshLock(s, "a");
      expect(await sm.refreshLockHeld(s)).toBe(false);
    });
  });
}
