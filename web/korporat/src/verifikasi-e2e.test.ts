import { afterAll, beforeAll, beforeEach, describe, expect, test } from "vitest";
import { cookieNames, createBffCore, createBffRoutes, loadBffConfig, type BffCore, type BffRoutes } from "@rs/bff-auth";
import { BffBrowser, FakeApi, ORIGIN, expectNoSecrets, json } from "../../packages/bff-auth/src/testing/bff-harness.ts";
import { FakeOp } from "../../packages/bff-auth/src/testing/fake-op.ts";
import { TestClock, testEnv, testSecrets } from "../../packages/bff-auth/src/testing/fixtures.ts";
import { MemoryBackend } from "../../packages/bff-auth/src/testing/memory-backend.ts";
import { BFF_APP, OPSI_ROUTES } from "./lib/server/bff.ts";
import { buatKlienVerifikasi, type KlienVerifikasi, type Ringkasan } from "./lib/verifikasi/klien.ts";

/**
 * F1-T103 e2e sisi server: klien verifikasi yang sama dengan yang dipakai browser memanggil
 * `/bff/v1/corporate/documents...` dan `/bff/v1/corporate/verification-resubmissions` lewat Route Handler BFF
 * `korporat` (cookie sesi, CSRF, proxy, kebijakan sesi) ke API Rust tiruan. Unggahan langsung ke object storage
 * memakai fetch tiruan (tanpa cookie, tanpa CSRF). Pola sama dengan `hr-e2e.test.ts`; tanpa browser sungguhan
 * (kerangka e2e peramban belum ada di repo).
 */
const MENIT = 60_000;
const CLIENT_ID = "rs-korporat-bff";
const SLOT = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
const secrets = testSecrets();
const op = new FakeOp({ clientId: CLIENT_ID, clientSecret: secrets.oidcClientSecret.expose(), redirectUri: `${ORIGIN}/auth/callback` });
const api = new FakeApi();
const { session: SID } = cookieNames(BFF_APP);

let clock: TestClock;
let backend: MemoryBackend;
let core: BffCore;
let routes: BffRoutes;
let browser: BffBrowser;
let unggahan: { url: string; init: RequestInit | undefined }[];
let setelahUnggah: () => void;
/** Jawaban object storage tiruan; bawaan 204. */
let jawabStorage: () => Promise<Response>;

beforeAll(async () => {
  await op.start();
  await api.start();
});
afterAll(async () => {
  await op.stop();
  await api.stop();
});
beforeEach(async () => {
  clock = new TestClock(Date.now());
  backend = new MemoryBackend(clock.now);
  core = await createBffCore(
    loadBffConfig(
      testEnv({
        BFF_APP_NAME: BFF_APP,
        BFF_OIDC_ISSUER: op.issuer,
        BFF_OIDC_CLIENT_ID: CLIENT_ID,
        BFF_API_BASE_URL: `${api.url}/`,
        BFF_OPENBAO_KV_PATH: "rs/bff/korporat",
      }),
    ),
    { secrets, backend, now: clock.now },
  );
  routes = createBffRoutes(() => Promise.resolve(core), OPSI_ROUTES);
  browser = new BffBrowser();
  api.requests.length = 0;
  api.next = null;
  unggahan = [];
  setelahUnggah = () => undefined;
  jawabStorage = () => Promise.resolve(new Response(null, { status: 204 }));
});

interface OpsiMasuk {
  /** Token CSRF yang dikirim klien; bawaan token sesi yang benar. */
  readonly csrf?: (benar: string) => string;
  /** Batas unggah langsung (test timeout). */
  readonly batasUnggahMs?: number;
}

async function masuk(o: OpsiMasuk = {}): Promise<KlienVerifikasi> {
  const r1 = await browser.send(routes.login.GET, "/auth/login");
  const search = op.authorize(new URL(r1.res.headers.get("location") ?? ""));
  await browser.send(routes.callback.GET, `/auth/callback${search}`);
  const st = await browser.send(routes.session.GET, "/auth/session");
  const benar = (json(st.body)["data"] as { csrf_token: string }).csrf_token;
  // Jembatan fetch: permintaan klien diteruskan ke Route Handler BFF seperti dari browser same-origin, beserta badannya.
  const jembatan = async (r: Request): Promise<Response> => {
    const u = new URL(r.url);
    const handler = routes.proxy[r.method as "GET" | "POST"];
    const headers: Record<string, string> = {};
    const csrf = r.headers.get("x-csrf-token");
    if (csrf !== null && csrf !== "") headers["x-csrf-token"] = csrf;
    const ct = r.headers.get("content-type");
    if (ct !== null) headers["content-type"] = ct;
    if (r.method !== "GET") headers["origin"] = ORIGIN;
    const badan = r.method === "GET" ? undefined : await r.text();
    const sent = await browser.send(handler, `${u.pathname}${u.search}`, {
      method: r.method,
      headers,
      ...(badan === undefined || badan === "" ? {} : { body: badan }),
    });
    return new Response(sent.body, { status: sent.res.status, headers: sent.res.headers });
  };
  const fetchUnggah = ((url: string, init?: RequestInit) => {
    unggahan.push({ url, init });
    setelahUnggah();
    return jawabStorage();
  }) as typeof fetch;
  return buatKlienVerifikasi(o.csrf === undefined ? benar : o.csrf(benar), {
    basis: `${ORIGIN}/bff`,
    fetch: jembatan as typeof fetch,
    fetchUnggah,
    ...(o.batasUnggahMs === undefined ? {} : { batasUnggahMs: o.batasUnggahMs }),
  });
}

const amplop = (data: unknown) => JSON.stringify({ response_code: "2000000", response_description: "Berhasil", data });
const galatApi = (status: number, kode: string) => ({
  status,
  headers: { "content-type": "application/problem+json" },
  body: JSON.stringify({ type: `https://problems.ruang-sehat.invalid/${kode}`, response_code: kode, response_description: "x", correlation_id: "c" }),
});
const RINGKASAN: Ringkasan = {
  status: "rejected",
  submission_round: 1,
  complete: true,
  rejection_reason: "authority_proof_insufficient",
  company: {
    legal_name: "PT Contoh Sintetis",
    nib: "1234567890123",
    npwp: "0123456789012345",
    office_address: "Jl. Contoh No. 1",
    office_phone: "021-0000000",
    submitted_at: "2027-03-01T01:15:00Z",
  },
  documents: [
    { kind: "nib", uploaded: true, uploaded_at: "2027-03-01T01:00:00Z" },
    { kind: "npwp", uploaded: true, uploaded_at: "2027-03-01T01:00:00Z" },
    { kind: "authority_proof", uploaded: true, uploaded_at: "2027-03-01T01:00:00Z" },
  ],
};
const sesiAda = (): boolean => browser.cookies.has(SID);
const foto = () => new File([new Uint8Array([0xff, 0xd8, 0xff, 0xe0])], "nib.jpg", { type: "image/jpeg" });

describe("e2e web korporat: status verifikasi dan dokumen lewat BFF (US-1901 AC2/AC6, US-1902 AC3)", () => {
  test("muat ringkasan: GET /v1/corporate/documents dengan token sesi; token tidak sampai browser", async () => {
    const klien = await masuk();
    api.next = { body: amplop(RINGKASAN) };
    const h = await klien.muat();
    expect(h.ok && h.data.jenis === "dokumen" && h.data.ringkasan.rejection_reason).toBe("authority_proof_insufficient");
    expect(api.last()?.method).toBe("GET");
    expect(api.last()?.path).toBe("/v1/corporate/documents");
    expect(api.last()?.headers.authorization).toMatch(/^Bearer /);
    expectNoSecrets(browser.responses, op.issuedSecrets);
  });

  test("perusahaan sudah Aktif (409 4090112) → status Aktif, sesi tetap hidup", async () => {
    const klien = await masuk();
    api.next = galatApi(409, "4090112");
    expect(await klien.muat()).toEqual({ ok: true, data: { jenis: "aktif" } });
    expect(sesiAda()).toBe(true);
  });

  test("unggah: slot lewat BFF → isi berkas langsung ke URL bertanda tangan tanpa cookie → konfirmasi lewat BFF", async () => {
    const klien = await masuk();
    api.next = {
      status: 201,
      body: amplop({ upload_slot_id: SLOT, expires_at: "2027-03-01T01:20:00Z", method: "put", url: "https://unggah.contoh.test/o?sig=rahasia", fields: [], headers: [{ name: "content-type", value: "image/jpeg" }] }),
    };
    // Jawaban konfirmasi disiapkan saat isi berkas "diunggah" (di antara permintaan slot dan konfirmasi).
    setelahUnggah = () => {
      api.next = { status: 201, body: amplop({ ...RINGKASAN, status: "pending_verification", rejection_reason: null }) };
    };
    const berkas = foto();
    const h = await klien.unggah("nib", berkas);
    expect(h.ok && h.data.status).toBe("pending_verification");
    const [slot, konfirmasi] = api.requests.slice(-2);
    expect(slot?.path).toBe("/v1/corporate/document-upload-slots");
    expect(JSON.parse(slot?.body ?? "{}")).toEqual({ content_type: "image/jpeg", size_bytes: berkas.size });
    expect(konfirmasi?.path).toBe("/v1/corporate/documents");
    expect(JSON.parse(konfirmasi?.body ?? "{}")).toEqual({ document_kind: "nib", upload_slot_id: SLOT });
    expect(unggahan).toHaveLength(1);
    expect(unggahan[0]?.url).toBe("https://unggah.contoh.test/o?sig=rahasia");
    expect(unggahan[0]?.init?.method).toBe("PUT");
    expect(unggahan[0]?.init?.credentials).toBe("omit");
    expect(unggahan[0]?.init?.referrerPolicy).toBe("no-referrer");
    expect(new Headers(unggahan[0]?.init?.headers).get("x-csrf-token")).toBeNull();
    expect(new Headers(unggahan[0]?.init?.headers).get("content-type")).toBe("image/jpeg");
  });

  test("negatif: PDF ditolak 415 saat slot → tidak ada unggahan ke object storage, pesan pindaian/foto", async () => {
    const klien = await masuk();
    api.next = galatApi(415, "4150001");
    const pdf = new File([new Uint8Array([0x25, 0x50, 0x44, 0x46])], "nib.pdf", { type: "application/pdf" });
    const h = await klien.unggah("nib", pdf);
    expect(h).toMatchObject({ ok: false, status: 415 });
    expect(!h.ok && h.pesan).toMatch(/pindaian atau foto/);
    expect(unggahan).toHaveLength(0);
    expect(sesiAda()).toBe(true);
  });

  test("ajukan ulang: POST /v1/corporate/verification-resubmissions, status kembali Menunggu verifikasi", async () => {
    const klien = await masuk();
    api.next = { body: amplop({ ...RINGKASAN, status: "pending_verification", submission_round: 2, rejection_reason: null }) };
    const h = await klien.ajukanUlang();
    expect(h.ok && h.data.status).toBe("pending_verification");
    expect(api.last()?.method).toBe("POST");
    expect(api.last()?.path).toBe("/v1/corporate/verification-resubmissions");
  });

  test.each([
    ["4090115", /Dokumen belum lengkap/],
    ["4090001", /sudah menunggu verifikasi/],
    ["4090112", /sudah Aktif/],
  ])("negatif: ajukan ulang 409 %s sampai ke klien dengan pesan sesuai kode, sesi tetap hidup", async (kode, pesan) => {
    const klien = await masuk();
    api.next = galatApi(409, kode);
    const h = await klien.ajukanUlang();
    expect(h).toMatchObject({ ok: false, status: 409, kode });
    expect(!h.ok && h.pesan).toMatch(pesan);
    expect(sesiAda()).toBe(true);
  });

  test("negatif: token CSRF hilang → 403 4030101 dari BFF, tidak diteruskan ke API", async () => {
    const klien = await masuk({ csrf: () => "" });
    const n = api.requests.length;
    const h = await klien.ajukanUlang();
    expect(h).toMatchObject({ ok: false, status: 403, kode: "4030101" });
    expect(!h.ok && h.pesan).toMatch(/Muat ulang halaman/);
    expect(api.requests.length).toBe(n);
  });

  test("negatif: token CSRF salah pada unggah → 403 4030101, slot tidak diminta dan tidak ada unggahan", async () => {
    const klien = await masuk({ csrf: (b) => `${b}x` });
    const n = api.requests.length;
    const h = await klien.unggah("nib", foto());
    expect(h).toMatchObject({ ok: false, status: 403, kode: "4030101" });
    expect(api.requests.length).toBe(n);
    expect(unggahan).toHaveLength(0);
  });

  test("negatif: sesi kedaluwarsa (tidak aktif 30 menit) → 401, sesi server terhapus, API tidak dipanggil", async () => {
    const klien = await masuk();
    const sid = browser.cookies.get(SID) ?? "";
    const n = api.requests.length;
    clock.advance(30 * MENIT);
    expect(await klien.muat()).toMatchObject({ ok: false, status: 401 });
    expect(await klien.ajukanUlang()).toMatchObject({ ok: false, status: 401 });
    expect(api.requests.length).toBe(n);
    expect(backend.keys().some((k) => k === core.sessions.key("s", sid))).toBe(false);
    expect(sesiAda()).toBe(false);
  });

  test("negatif: unduh dokumen yang belum diunggah (404) sampai ke klien; URL unduh tidak pernah dari BFF lain", async () => {
    const klien = await masuk();
    api.next = galatApi(404, "4040001");
    expect(await klien.urlUnduh("authority_proof")).toMatchObject({ ok: false, status: 404 });
    expect(api.last()?.path).toBe("/v1/corporate/document-download-urls");
    expect(JSON.parse(api.last()?.body ?? "{}")).toEqual({ document_kind: "authority_proof" });
  });
  const slotJawaban = (method: "put" | "post") => ({
    status: 201,
    body: amplop({
      upload_slot_id: SLOT,
      expires_at: "2027-03-01T01:20:00Z",
      method,
      url: "https://unggah.contoh.test/bucket",
      fields: method === "post" ? [{ name: "key", value: "k1" }, { name: "policy", value: "p1" }] : [],
      headers: [],
    }),
  });
  const adaKonfirmasi = (): boolean => api.requests.some((r) => r.path === "/v1/corporate/documents" && r.method === "POST");

  test.each([401, 403, 500])(
    "negatif: object storage menjawab %i → status 0 (bukan sesi berakhir/akses ditolak), tanpa konfirmasi, sesi tetap ada",
    async (kodeStorage) => {
      const klien = await masuk();
      api.next = slotJawaban("put");
      jawabStorage = () => Promise.resolve(new Response("<Error/>", { status: kodeStorage }));
      const h = await klien.unggah("nib", foto());
      expect(h).toMatchObject({ ok: false, status: 0, kode: null });
      expect(!h.ok && h.pesan).toMatch(/Berkas gagal diunggah/);
      expect(adaKonfirmasi()).toBe(false);
      expect(sesiAda()).toBe(true);
    },
  );

  test("jalur POST formulir: field slot ditambahkan sebelum `file`, tanpa cookie dan tanpa referrer", async () => {
    const klien = await masuk();
    api.next = slotJawaban("post");
    setelahUnggah = () => {
      api.next = { status: 201, body: amplop(RINGKASAN) };
    };
    const berkas = foto();
    const h = await klien.unggah("npwp", berkas);
    expect(h.ok).toBe(true);
    const init = unggahan[0]?.init;
    expect(init?.method).toBe("POST");
    expect(init?.credentials).toBe("omit");
    expect(init?.referrerPolicy).toBe("no-referrer");
    const f = init?.body as FormData;
    expect([...f.keys()]).toEqual(["key", "policy", "file"]);
    expect(f.get("key")).toBe("k1");
    expect((f.get("file") as File).name).toBe("nib.jpg");
    expect(JSON.parse(api.last()?.body ?? "{}")).toEqual({ document_kind: "npwp", upload_slot_id: SLOT });
  });

  test("negatif: unggah ke storage gagal jaringan → 'Gagal terhubung', tanpa konfirmasi", async () => {
    const klien = await masuk();
    api.next = slotJawaban("put");
    jawabStorage = () => Promise.reject(new TypeError("fetch failed"));
    const h = await klien.unggah("nib", foto());
    expect(h).toMatchObject({ ok: false, status: 0 });
    expect(!h.ok && h.pesan).toMatch(/Gagal terhubung/);
    expect(adaKonfirmasi()).toBe(false);
    expect(sesiAda()).toBe(true);
  });

  test("negatif: unggah melewati batas waktu → dibatalkan, 'Gagal terhubung', tanpa konfirmasi", async () => {
    const klien = await masuk({ batasUnggahMs: 20 });
    api.next = slotJawaban("put");
    let sinyal: AbortSignal | null | undefined;
    jawabStorage = () =>
      new Promise<Response>((_terima, tolak) => {
        sinyal = unggahan[0]?.init?.signal;
        sinyal?.addEventListener("abort", () => {
          tolak(new DOMException("timeout", "TimeoutError"));
        });
      });
    const h = await klien.unggah("nib", foto());
    expect(sinyal?.aborted).toBe(true);
    expect(!h.ok && h.pesan).toMatch(/Gagal terhubung/);
    expect(adaKonfirmasi()).toBe(false);
  });
});
