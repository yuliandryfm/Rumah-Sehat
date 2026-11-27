import { describe, expect, test } from "vitest";
import { buatCsp, originUnggahDariEnv } from "../../keamanan.ts";
import { OPSI_ROUTES } from "../server/bff.ts";
import { menuUntuk } from "../navigasi.ts";
import { buatKlienCms, galat } from "./klien.ts";
import { bolehDiarsipkan, bolehDihapus, bolehDijadwalkan, bolehDiubah, formatWaktu } from "./model.ts";
import { SANDBOX_PRATINJAU, dokumenPratinjau, escapeHtml } from "./pratinjau.ts";

const PAYLOAD_XSS = [
  "<script>alert(1)</script>",
  '"><img src=x onerror=alert(1)>',
  "[klik](javascript:alert(1))",
  "<iframe srcdoc='<script>top.location=1</script>'></iframe>",
  "</pre><script>fetch('/bff/backoffice/v1/contents')</script>",
];

describe("dokumen pratinjau", () => {
  test.each(PAYLOAD_XSS)("payload XSS dalam isi menjadi teks, bukan markup: %s", (p) => {
    const doc = dokumenPratinjau("Judul", p);
    // Satu-satunya tag di dokumen adalah milik kerangka; tidak ada tag dari payload.
    expect(doc.match(/<script|<img|<iframe|<a /gi)).toBeNull();
    expect(doc).toContain(escapeHtml(p));
  });

  test("payload di judul di-escape juga", () => {
    const doc = dokumenPratinjau("<script>x</script>", "isi");
    expect(doc).not.toContain("<script");
    expect(doc).toContain("&lt;script&gt;");
  });

  test("sandbox kosong: tanpa allow-same-origin dan tanpa allow-scripts", () => {
    expect(SANDBOX_PRATINJAU).toBe("");
    expect(SANDBOX_PRATINJAU).not.toContain("allow-same-origin");
    expect(SANDBOX_PRATINJAU).not.toContain("allow-scripts");
  });

  test("escapeHtml menangani lima karakter berbahaya", () => {
    expect(escapeHtml(`&<>"'`)).toBe("&amp;&lt;&gt;&quot;&#39;");
  });
});

describe("aturan status dan model", () => {
  test("hanya draf yang boleh diubah dan dijadwalkan; arsip untuk draf dan terjadwal; terhapus tidak dihapus lagi", () => {
    expect(bolehDiubah({ display_status: "draft" })).toBe(true);
    expect(bolehDijadwalkan({ display_status: "draft" })).toBe(true);
    for (const s of ["scheduled", "live", "ended", "archived", "deleted"] as const) {
      expect(bolehDiubah({ display_status: s })).toBe(false);
    }
    expect(bolehDiarsipkan({ display_status: "scheduled" })).toBe(true);
    expect(bolehDiarsipkan({ display_status: "live" })).toBe(false);
    expect(bolehDihapus({ display_status: "deleted" })).toBe(false);
    expect(bolehDihapus({ display_status: "live" })).toBe(true);
  });

  test("formatWaktu: kosong dan tidak valid menjadi tanda hubung", () => {
    expect(formatWaktu(null)).toBe("-");
    expect(formatWaktu("bukan-tanggal")).toBe("-");
    expect(formatWaktu("2027-03-01T01:00:00Z")).toContain("2027");
  });
});

describe("navigasi dan BFF", () => {
  test("menu Konten hanya untuk sub-peran Konten", () => {
    expect(menuUntuk(["Konten"]).map((m) => m.href)).toContain("/konten");
    for (const s of ["Super Admin", "Verifikator", "Layanan Pelanggan", "Keuangan", "Auditor", "Petugas PDP"]) {
      expect(menuUntuk([s]).map((m) => m.href)).not.toContain("/konten");
    }
    expect(menuUntuk([]).map((m) => m.href)).not.toContain("/konten");
  });

  test("proxy BFF hanya meneruskan namespace backoffice/v1/", () => {
    expect(OPSI_ROUTES.allowedApiPrefixes).toEqual(["backoffice/v1/"]);
  });
});

describe("CSP unggah gambar", () => {
  test("origin unggah hanya masuk connect-src; unsafe-* tetap tidak ada", () => {
    const csp = buatCsp("n", false, "https://neo.example.test");
    expect(csp).toContain("connect-src 'self' https://neo.example.test;");
    expect(csp).not.toMatch(/unsafe-/);
    expect(buatCsp("n")).toContain("connect-src 'self';");
  });

  test.each([
    [undefined, false, null],
    ["", false, null],
    ["bukan url", false, null],
    ["http://neo.example.test", false, null],
    ["http://127.0.0.1:18333", true, "http://127.0.0.1:18333"],
    ["https://neo.example.test", false, "https://neo.example.test"],
    ["https://neo.example.test/bucket", false, null],
    ["https://u:p@neo.example.test", false, null],
    ["https://neo.example.test/?x=1", false, null],
    ["javascript:alert(1)", true, null],
    // Review T1 F1-T103: host longgar dan pemangkasan diam-diam parser URL ditolak.
    ["https://*", false, null],
    ["https://*.x.test", false, null],
    ["https://a.test'x'", false, null],
    ["https://a,test", false, null],
    ["https://a;b.test", false, null],
    ["https://neo.exa\tmple.test", false, null],
    ["https://neo.example.test\\", false, null],
    ["https://neo.example.test/%2e", false, null],
    ["https://neo.example.test/", false, "https://neo.example.test"],
    ["  https://neo.example.test  ", false, "https://neo.example.test"],
  ])("originUnggahDariEnv(%s, dev=%s) = %s", (nilai, dev, harapan) => {
    expect(originUnggahDariEnv(nilai, dev)).toBe(harapan);
  });
});

describe("galat API", () => {
  test("pesan per status memakai teks bantu; field error terbaca; isi aneh tidak melempar", () => {
    expect(galat(403, null).pesan).toContain("sub-peran Konten");
    const g = galat(422, {
      response_code: "4220001",
      response_description: "Isian tidak valid",
      errors: [{ field: "body", code: "raw_html", message: "HTML mentah tidak diizinkan" }, null, { field: 1 }],
    });
    expect(g.pesan).toBe("Isian tidak valid");
    expect(g.isian).toEqual([{ field: "body", code: "raw_html", message: "HTML mentah tidak diizinkan" }]);
    expect(g.kode).toBe("4220001");
    expect(galat(500, "teks").pesan).toBe("Permintaan gagal. Coba lagi.");
  });
});

const json = (status: number, badan: unknown) =>
  new Response(JSON.stringify(badan), { status, headers: { "content-type": status < 400 ? "application/json" : "application/problem+json" } });

describe("klien CMS lewat BFF", () => {
  test("daftar: path /bff/backoffice/v1/contents, saringan, dan X-CSRF-Token; tanpa Authorization", async () => {
    const dilihat: Request[] = [];
    const klien = buatKlienCms("csrf-uji", {
      basis: "http://bo.test/bff",
      fetch: (r) => {
        dilihat.push(r as Request);
        return Promise.resolve(json(200, { data: [{ content_id: "a" }], meta: { has_more: true, next_cursor: "c1" } }));
      },
    });
    const h = await klien.daftar({ status: "deleted", channel: "site", site_page: "about" });
    expect(h.ok && h.data.items.length).toBe(1);
    expect(h.ok && h.data.nextCursor).toBe("c1");
    const u = new URL(dilihat[0]?.url ?? "");
    expect(u.pathname).toBe("/bff/backoffice/v1/contents");
    expect(u.searchParams.get("status")).toBe("deleted");
    expect(u.searchParams.get("channel")).toBe("site");
    expect(u.searchParams.get("site_page")).toBe("about");
    expect(dilihat[0]?.headers.get("x-csrf-token")).toBe("csrf-uji");
    expect(dilihat[0]?.headers.get("authorization")).toBeNull();
  });

  test("403 dan jaringan putus menjadi Hasil gagal, tanpa melempar", async () => {
    const k403 = buatKlienCms("c", { basis: "http://bo.test/bff", fetch: () => Promise.resolve(json(403, { response_code: "4030001" })) });
    const a = await k403.ambil("0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b");
    expect(a.ok).toBe(false);
    expect(!a.ok && a.status).toBe(403);
    const kPutus = buatKlienCms("c", { basis: "http://bo.test/bff", fetch: () => Promise.reject(new Error("putus")) });
    const b = await kPutus.buat({ title: "x", channel: "site", content_type: "info_page" });
    expect(!b.ok && b.status).toBe(503);
  });

  test("hapus mengirim delete_reason; jadwal mengirim waktu apa adanya", async () => {
    const badan: unknown[] = [];
    const klien = buatKlienCms("c", {
      basis: "http://bo.test/bff",
      fetch: async (r) => {
        badan.push([new URL((r as Request).url).pathname, await (r as Request).json()]);
        return json(200, { data: { content_id: "a" } });
      },
    });
    const id = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
    await klien.hapus(id, "konten usang");
    await klien.jadwalkan(id, "2027-03-01T08:00:00+07:00", null);
    expect(badan).toEqual([
      [`/bff/backoffice/v1/contents/${id}/delete`, { delete_reason: "konten usang" }],
      [`/bff/backoffice/v1/contents/${id}/schedule`, { publish_start_at: "2027-03-01T08:00:00+07:00", publish_end_at: null }],
    ]);
  });

  test("unggah gambar: slot, unggah tanpa kredensial ke URL slot, lalu pasang dengan teks alternatif", async () => {
    const id = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
    const urutan: string[] = [];
    const tujuanUnggah: { url: string; init: RequestInit | undefined }[] = [];
    const klien = buatKlienCms("c", {
      basis: "http://bo.test/bff",
      fetch: async (r) => {
        const req = r as Request;
        const p = new URL(req.url).pathname;
        urutan.push(`${req.method} ${p}`);
        if (p.endsWith("/upload-slots")) {
          return json(201, {
            data: { upload_slot_id: "s1", method: "put", url: "https://neo.example.test/x?sig=1", fields: [], headers: [{ name: "content-type", value: "image/png" }], expires_at: "2027-01-01T00:00:00Z" },
          });
        }
        expect(await req.json()).toEqual({ upload_slot_id: "s1", image_alt_text: "Ilustrasi" });
        return json(200, { data: { content_id: id } });
      },
      fetchUnggah: (u, init) => {
        tujuanUnggah.push({ url: u as string, init });
        return Promise.resolve(new Response(null, { status: 200 }));
      },
    });
    const h = await klien.unggahGambar(id, new File([new Uint8Array(3)], "a.png", { type: "image/png" }), "Ilustrasi");
    expect(h.ok).toBe(true);
    expect(urutan).toEqual(["POST /bff/backoffice/v1/content-images/upload-slots", `PUT /bff/backoffice/v1/contents/${id}/image`]);
    expect(tujuanUnggah[0]?.url).toBe("https://neo.example.test/x?sig=1");
    expect(tujuanUnggah[0]?.init?.credentials).toBe("omit");
  });

  test("unggah gambar: slot ditolak (413) atau unggah gagal menghentikan alur sebelum pasang", async () => {
    const id = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
    const berkas = new File([new Uint8Array(3)], "a.png", { type: "image/png" });
    const k413 = buatKlienCms("c", { basis: "http://bo.test/bff", fetch: () => Promise.resolve(json(413, {})) });
    const a = await k413.unggahGambar(id, berkas, "x");
    expect(!a.ok && a.status).toBe(413);

    let dipasang = false;
    const kGagal = buatKlienCms("c", {
      basis: "http://bo.test/bff",
      fetch: (r) => {
        if ((r as Request).url.endsWith("/image")) dipasang = true;
        return Promise.resolve(json(201, { data: { upload_slot_id: "s", method: "post", url: "https://neo.example.test/", fields: [{ name: "k", value: "v" }], headers: [], expires_at: "2027-01-01T00:00:00Z" } }));
      },
      fetchUnggah: () => Promise.resolve(new Response(null, { status: 403 })),
    });
    const b = await kGagal.unggahGambar(id, berkas, "x");
    expect(b.ok).toBe(false);
    expect(dipasang).toBe(false);
  });
});
