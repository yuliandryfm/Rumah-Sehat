// CR-028: permintaan pertama setelah start (dan yang serentak) tidak boleh gagal dengan
// "Stream isn't writeable and enableOfflineQueue options is false"; bila Valkey tak kunjung siap,
// galatnya terkendali (BackendUnavailableError, dijawab 503 oleh rute BFF).
import { createServer, type Server, type Socket } from "node:net";
import { Redis } from "ioredis";
import { afterEach, describe, expect, test, vi } from "vitest";
import { Sensitive } from "./sensitive.ts";
import { BackendUnavailableError, ValkeyBackend } from "./store.ts";

const servers: Server[] = [];
const backends: ValkeyBackend[] = [];
const soket: Socket[] = [];

afterEach(async () => {
  vi.restoreAllMocks();
  await Promise.all(backends.splice(0).map((b) => b.close()));
  for (const k of soket.splice(0)) {
    k.destroy();
  }
  await Promise.all(servers.splice(0).map((s) => new Promise<void>((r) => s.close(() => { r(); }))));
});

/** Server RESP minimal: balasan INFO (siap), SET → OK; setiap perintah lain → OK. Jawaban pertama ditunda. */
async function valkeyTiruan(tundaMs: number): Promise<URL> {
  const server = createServer((sock: Socket) => {
    soket.push(sock);
    let aktif = false;
    setTimeout(() => { aktif = true; sock.emit("data", Buffer.alloc(0)); }, tundaMs);
    let sisa = "";
    sock.on("data", (d: Buffer) => {
      sisa += d.toString("latin1");
      if (!aktif) {
        return;
      }
      const perintah = sisa.split(/\*\d+\r\n/).filter((x) => x.length > 0);
      sisa = "";
      for (const p of perintah) {
        sock.write(/\binfo\b/i.test(p) ? "$9\r\nloading:0\r\n" : "+OK\r\n");
      }
    });
    sock.on("error", () => undefined);
  });
  servers.push(server);
  await new Promise<void>((r) => server.listen(0, "127.0.0.1", r));
  const addr = server.address();
  if (addr === null || typeof addr === "string") {
    throw new Error("alamat server tiruan tidak tersedia");
  }
  return new URL(`redis://127.0.0.1:${String(addr.port)}`);
}

function buat(url: URL, timeoutMs: number): ValkeyBackend {
  const b = new ValkeyBackend({ url, password: new Sensitive("x"), timeoutMs });
  backends.push(b);
  return b;
}

describe("ValkeyBackend menunggu koneksi siap (CR-028)", () => {
  test("permintaan pertama dan serentak setelah start menunggu koneksi, bukan gagal", async () => {
    const url = await valkeyTiruan(300);
    const b = buat(url, 5_000);
    const hasil = await Promise.all([1, 2, 3, 4, 5].map((i) => b.putOnce(`k${String(i)}`, "v", 1000)));
    expect(hasil).toEqual([true, true, true, true, true]);
    // Setelah siap, perintah berikutnya langsung jalan.
    expect(await b.putOnce("k6", "v", 1000)).toBe(true);
  });

  test("Valkey tak terjangkau: galat terkendali dalam batas waktu, bukan galat antrean offline", async () => {
    const tutup = createServer();
    await new Promise<void>((r) => tutup.listen(0, "127.0.0.1", r));
    const addr = tutup.address();
    if (addr === null || typeof addr === "string") {
      throw new Error("alamat tidak tersedia");
    }
    const port = addr.port;
    await new Promise<void>((r) => tutup.close(() => { r(); }));
    const b = buat(new URL(`redis://127.0.0.1:${String(port)}`), 1_000);
    const mulai = Date.now();
    const galat = await b.putOnce("k", "v", 1000).then(() => null, (e: unknown) => e);
    expect(galat).toBeInstanceOf(BackendUnavailableError);
    expect((galat as Error).message).not.toMatch(/writeable|offlineQueue/i);
    expect(Date.now() - mulai).toBeLessThan(4_000);
    // Permintaan serentak juga mendapat galat terkendali yang sama.
    const lain = await Promise.all([b.exists("a"), b.exists("b")].map((p) => p.then(() => null, (e: unknown) => e)));
    for (const e of lain) {
      expect(e).toBeInstanceOf(BackendUnavailableError);
    }
  });

  test("Valkey menggantung (menerima koneksi, tak menjawab): BackendUnavailableError setelah batas waktu", async () => {
    const url = await valkeyTiruan(60_000);
    const b = buat(url, 500);
    await expect(b.getHash("k")).rejects.toBeInstanceOf(BackendUnavailableError);
  });

  test("timeout berulang tidak menambah pendengar ready/end pada koneksi (tanpa kebocoran)", async () => {
    const dengar = vi.spyOn(Redis.prototype, "once");
    const b = buat(new URL("redis://127.0.0.1:1"), 150);
    for (let i = 0; i < 15; i++) {
      await expect(b.getHash("k")).rejects.toBeInstanceOf(BackendUnavailableError);
    }
    const klien = dengar.mock.contexts[0] as Redis;
    expect(klien.listenerCount("ready")).toBe(0);
    expect(klien.listenerCount("end")).toBe(0);
  });
});
