// Harness test Route Handler BFF (KHUSUS TEST): browser tiruan dengan cookie jar, API Rust tiruan, dan
// pemeriksa kebocoran token. Dipakai test unit (authorization server tiruan) dan test integrasi Keycloak.

import { createServer, type IncomingMessage, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { expect } from "vitest";
import type { BffHandler } from "../routes.ts";

export const ORIGIN = "http://127.0.0.1:3000";

export interface UpstreamRequest {
  readonly method: string;
  readonly path: string;
  readonly headers: IncomingMessage["headers"];
  readonly body: string;
}

export interface UpstreamReply {
  status?: number;
  headers?: Record<string, string | string[]>;
  body?: string;
  delayMs?: number;
}

/** API Rust tiruan: mencatat permintaan dan membalas sesuai `next` (bawaan 200 envelope). */
export class FakeApi {
  readonly requests: UpstreamRequest[] = [];
  next: UpstreamReply | null = null;
  url = "";
  #server: Server | null = null;

  async start(): Promise<void> {
    this.#server = createServer((req, res) => {
      const chunks: Buffer[] = [];
      req.on("data", (c: Buffer) => chunks.push(c));
      req.on("end", () => {
        this.requests.push({
          method: req.method ?? "",
          path: req.url ?? "",
          headers: req.headers,
          body: Buffer.concat(chunks).toString("utf8"),
        });
        const reply = this.next ?? {};
        const send = (): void => {
          res.statusCode = reply.status ?? 200;
          for (const [k, v] of Object.entries(
            reply.headers ?? { "content-type": "application/json", "x-correlation-id": String(req.headers["x-correlation-id"]) },
          )) {
            res.setHeader(k, v);
          }
          res.end(reply.body ?? JSON.stringify({ response_code: "2000000", response_description: "Berhasil", data: { ok: true } }));
        };
        if (reply.delayMs !== undefined) {
          setTimeout(send, reply.delayMs);
        } else {
          send();
        }
      });
    });
    await new Promise<void>((resolve) => this.#server?.listen(0, "127.0.0.1", resolve));
    this.url = `http://127.0.0.1:${String((this.#server.address() as AddressInfo).port)}`;
  }

  last(): UpstreamRequest | undefined {
    return this.requests.at(-1);
  }

  async stop(): Promise<void> {
    await new Promise<void>((resolve) => {
      if (this.#server === null) {
        resolve();
        return;
      }
      this.#server.closeAllConnections();
      this.#server.close(() => {
        resolve();
      });
    });
  }
}

export interface Sent {
  readonly res: Response;
  readonly body: string;
  readonly setCookies: string[];
}

/** Browser tiruan untuk BFF: menyimpan cookie dari Set-Cookie dan mengirimnya kembali. */
export class BffBrowser {
  readonly cookies = new Map<string, string>();
  readonly responses: Sent[] = [];

  async send(
    handler: BffHandler,
    path: string,
    init: { method?: string; headers?: Record<string, string>; body?: string; noCookies?: boolean } = {},
  ): Promise<Sent> {
    const headers = new Headers(init.headers);
    if (init.noCookies !== true && this.cookies.size > 0) {
      headers.set("cookie", [...this.cookies].map(([k, v]) => `${k}=${v}`).join("; "));
    }
    const req = new Request(`${ORIGIN}${path}`, {
      method: init.method ?? "GET",
      headers,
      ...(init.body === undefined ? {} : { body: init.body }),
    });
    const res = await handler(req);
    const setCookies = res.headers.getSetCookie();
    for (const c of setCookies) {
      const [pair = ""] = c.split(";");
      const i = pair.indexOf("=");
      const name = pair.slice(0, i);
      const value = pair.slice(i + 1);
      if (value === "" || /;\s*Max-Age=0/i.test(c)) {
        this.cookies.delete(name);
      } else {
        this.cookies.set(name, value);
      }
    }
    const sent = { res, body: await res.text(), setCookies };
    this.responses.push(sent);
    return sent;
  }
}

/** Tidak satu pun rahasia muncul di status, header, atau badan respons mana pun yang diterima browser. */
export function expectNoSecrets(responses: readonly Sent[], secrets: readonly string[]): void {
  expect(secrets.length).toBeGreaterThan(0);
  for (const r of responses) {
    const headerText = [...r.res.headers].map(([k, v]) => `${k}: ${v}`).join("\n");
    for (const s of secrets) {
      expect(headerText.includes(s), `rahasia bocor di header respons`).toBe(false);
      expect(r.body.includes(s), `rahasia bocor di badan respons`).toBe(false);
    }
  }
}

export function json(body: string): Record<string, unknown> {
  return JSON.parse(body) as Record<string, unknown>;
}
