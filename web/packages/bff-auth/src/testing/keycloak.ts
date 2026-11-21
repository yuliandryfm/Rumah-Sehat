// Keycloak sekali pakai + realm `admin` sebagai kode (infra/keycloak/realms) untuk test integrasi BFF
// (KHUSUS TEST). Langkah sama dengan scripts/keycloak-check.sh mode ephemeral: image Keycloak dan
// keycloak-config-cli dari infra/local/compose.yaml, konfigurasi server infra/keycloak/keycloak.conf,
// kredensial admin dan secret klien acak sekali pakai. Alur browser (login, Configure OTP, kode pemulihan,
// OTP) meniru infra/keycloak/test/keycloak_check.py.

import { createHmac, randomBytes } from "node:crypto";
import { Container, REPO_ROOT, composeImage, dockerOk, randomSecret, waitFor } from "./containers.ts";

export const BACKOFFICE_REDIRECT = "http://127.0.0.1:3000/auth/callback";
/** > Quick Login Check 1000 ms realm (KEPUTUSAN-190): kiriman kredensial lebih cepat memicu penguncian. */
const JEDA_MS = 1200;

export class EphemeralKeycloak {
  readonly container: Container;
  readonly baseUrl: string;
  readonly backofficeSecret: string;
  readonly #adminUser = "admin-uji";
  readonly #adminPassword: string;

  private constructor(container: Container, baseUrl: string, backofficeSecret: string, adminPassword: string) {
    this.container = container;
    this.baseUrl = baseUrl;
    this.backofficeSecret = backofficeSecret;
    this.#adminPassword = adminPassword;
  }

  static async start(): Promise<EphemeralKeycloak> {
    const adminPassword = randomSecret();
    const backofficeSecret = randomSecret();
    const container = Container.run({
      image: composeImage("quay.io/keycloak/keycloak"),
      ports: [8080],
      env: { KC_BOOTSTRAP_ADMIN_USERNAME: "admin-uji", KC_BOOTSTRAP_ADMIN_PASSWORD: adminPassword },
      extra: [
        "-v",
        `${REPO_ROOT}infra/keycloak/keycloak.conf:/opt/keycloak/conf/keycloak.conf:ro`,
        "-v",
        `${REPO_ROOT}infra/keycloak/themes/ruang-sehat:/opt/keycloak/themes/ruang-sehat:ro`,
      ],
      cmd: ["start-dev"],
    });
    try {
      const baseUrl = `http://127.0.0.1:${String(container.hostPort(8080))}`;
      await waitFor(async () => (await fetch(`${baseUrl}/realms/master`)).status === 200, 240_000, "Keycloak");
      const realmEnv = {
        RS_KC_SMTP_HOST: "email-relay",
        RS_KC_SMTP_PORT: "2525",
        RS_KC_SMTP_FROM: "noreply@uji.invalid",
        RS_KC_PASIEN_MOBILE_REDIRECT_URI: "com.example.ruangsehat.pasien:/oauth2redirect",
        RS_KC_BACKOFFICE_BFF_REDIRECT_URI: BACKOFFICE_REDIRECT,
        RS_KC_PORTAL_BFF_REDIRECT_URI: "http://127.0.0.1:3001/auth/callback",
        RS_KC_KORPORAT_BFF_REDIRECT_URI: "http://127.0.0.1:3002/auth/callback",
      };
      const secretEnv = {
        KEYCLOAK_USER: "admin-uji",
        KEYCLOAK_PASSWORD: adminPassword,
        RS_KC_BACKOFFICE_BFF_SECRET: backofficeSecret,
        RS_KC_PORTAL_BFF_SECRET: randomSecret(),
        RS_KC_EVENT_READER_PASIEN_SECRET: randomSecret(),
        RS_KC_EVENT_READER_ADMIN_SECRET: randomSecret(),
        RS_KC_EVENT_READER_KORPORAT_SECRET: randomSecret(),
        RS_KC_KORPORAT_BFF_SECRET: randomSecret(),
        RS_KC_ADMIN_MANAGER_SECRET: randomSecret(),
      };
      const args = ["run", "--rm", "--network", `container:${container.id}`, "-v", `${REPO_ROOT}infra/keycloak/realms:/config:ro`];
      for (const [k, v] of Object.entries({
        KEYCLOAK_URL: "http://127.0.0.1:8080",
        KEYCLOAK_AVAILABILITYCHECK_ENABLED: "true",
        KEYCLOAK_AVAILABILITYCHECK_TIMEOUT: "120s",
        IMPORT_FILES_LOCATIONS: "/config/*.json",
        IMPORT_VARSUBSTITUTION_ENABLED: "true",
        IMPORT_CACHE_ENABLED: "false",
        ...realmEnv,
      })) {
        args.push("-e", `${k}=${v}`);
      }
      for (const k of Object.keys(secretEnv)) {
        args.push("-e", k);
      }
      args.push(composeImage("adorsys/keycloak-config-cli"));
      dockerOk(args, secretEnv, 300_000);
      return new EphemeralKeycloak(container, baseUrl, backofficeSecret, adminPassword);
    } catch (err) {
      const logs = container.logs();
      container.remove();
      throw new Error(`Keycloak sekali pakai gagal disiapkan: ${String(err)}\n${logs}`, { cause: err });
    }
  }

  stop(): void {
    this.container.remove();
  }

  async #adminToken(): Promise<string> {
    const res = await fetch(`${this.baseUrl}/realms/master/protocol/openid-connect/token`, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({
        grant_type: "password",
        client_id: "admin-cli",
        username: this.#adminUser,
        password: this.#adminPassword,
      }).toString(),
    });
    if (!res.ok) {
      throw new Error(`login admin master gagal: HTTP ${String(res.status)}`);
    }
    return ((await res.json()) as { access_token: string }).access_token;
  }

  /** Admin REST realm `admin`: `path` relatif terhadap `/admin/realms/admin/`. */
  async admin(method: string, path: string, body?: unknown): Promise<Response> {
    return fetch(`${this.baseUrl}/admin/realms/admin/${path}`, {
      method,
      headers: {
        Authorization: `Bearer ${await this.#adminToken()}`,
        ...(body === undefined ? {} : { "Content-Type": "application/json" }),
      },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
    });
  }

  async createUser(): Promise<TestUser> {
    const email = `f1t41-${randomBytes(4).toString("hex")}@uji.invalid`;
    const password = `Uji-${randomBytes(12).toString("base64url")}`;
    const res = await this.admin("POST", "users", {
      username: email,
      email,
      emailVerified: true,
      enabled: true,
      credentials: [{ type: "password", value: password, temporary: false }],
    });
    if (res.status !== 201) {
      throw new Error(`gagal membuat user uji: HTTP ${String(res.status)}`);
    }
    const id = (res.headers.get("location") ?? "").split("/").pop() ?? "";
    return new TestUser(id, email, password);
  }

  async userSessionCount(user: TestUser): Promise<number> {
    const res = await this.admin("GET", `users/${user.id}/sessions`);
    if (!res.ok) {
      throw new Error(`gagal membaca sesi user: HTTP ${String(res.status)}`);
    }
    return ((await res.json()) as unknown[]).length;
  }

  /** Admin mencabut semua sesi user (Admin REST "logout user"). */
  async logoutUser(user: TestUser): Promise<void> {
    const res = await this.admin("POST", `users/${user.id}/logout`);
    if (res.status !== 204) {
      throw new Error(`gagal mencabut sesi user: HTTP ${String(res.status)}`);
    }
  }
}

export class TestUser {
  otpKey: string | null = null;
  readonly #usedSlots = new Set<number>();

  constructor(
    readonly id: string,
    readonly email: string,
    readonly password: string,
  ) {}

  /**
   * Kode TOTP (HMAC-SHA1, 6 digit, 30 detik) dari slot yang belum dipakai (kode tidak dapat dipakai ulang).
   * Hanya slot sekarang dan berikutnya: kode dikirim ±1,2 detik kemudian (jeda Quick Login Check), sehingga
   * slot sebelumnya dapat sudah di luar jendela look-ahead 1 saat diterima Keycloak.
   */
  async otp(): Promise<string> {
    if (this.otpKey === null) {
      throw new Error("TOTP belum didaftarkan");
    }
    for (;;) {
      const now = Math.floor(Date.now() / 1000 / 30);
      for (const off of [0, 1]) {
        if (!this.#usedSlots.has(now + off)) {
          this.#usedSlots.add(now + off);
          return totp(this.otpKey, now + off);
        }
      }
      await new Promise((r) => setTimeout(r, (now + 1) * 30_000 - Date.now() + 500));
    }
  }
}

/** RFC 6238 TOTP SHA1. Kunci = teks `totpSecret` formulir Configure OTP apa adanya (lihat kc_http.py). */
export function totp(key: string, counter: number): string {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64BE(BigInt(counter));
  const mac = createHmac("sha1", Buffer.from(key, "utf8")).update(buf).digest();
  const o = (mac[mac.length - 1] ?? 0) & 0x0f;
  const code = (mac.readUInt32BE(o) & 0x7fffffff) % 1_000_000;
  return String(code).padStart(6, "0");
}

interface Page {
  readonly status: number;
  readonly location: string | null;
  readonly body: string;
}

function unescapeHtml(s: string): string {
  return s
    .replace(/&amp;/g, "&")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">");
}

function attr(tag: string, name: string): string | undefined {
  const m = new RegExp(`\\s${name}="([^"]*)"`).exec(tag);
  return m?.[1] === undefined ? undefined : unescapeHtml(m[1]);
}

/** Formulir HTML berdasarkan id: action + nilai input (pengurai sederhana untuk halaman Keycloak). */
export function form(body: string, id: string): { action: string; inputs: Record<string, string> } | null {
  const re = /<form\b[^>]*>/g;
  for (let m = re.exec(body); m !== null; m = re.exec(body)) {
    if (attr(m[0], "id") !== id) {
      continue;
    }
    const action = attr(m[0], "action") ?? "";
    const end = body.indexOf("</form>", m.index);
    const inner = body.slice(m.index, end === -1 ? undefined : end);
    const inputs: Record<string, string> = {};
    for (const t of inner.match(/<input\b[^>]*>/g) ?? []) {
      const name = attr(t, "name");
      if (name !== undefined) {
        inputs[name] = attr(t, "value") ?? "";
      }
    }
    return { action, inputs };
  }
  return null;
}

/** Browser minimal untuk halaman Keycloak: cookie per nama, tanpa mengikuti redirect otomatis. */
export class KcBrowser {
  readonly #cookies = new Map<string, string>();
  readonly #base: string;

  constructor(base: string) {
    this.#base = base;
  }

  async open(url: string, data?: Record<string, string>): Promise<Page> {
    const target = new URL(url, this.#base);
    const headers: Record<string, string> = {};
    if (this.#cookies.size > 0) {
      headers["Cookie"] = [...this.#cookies].map(([k, v]) => `${k}=${v}`).join("; ");
    }
    if (data !== undefined) {
      headers["Content-Type"] = "application/x-www-form-urlencoded";
    }
    const res = await fetch(target, {
      method: data === undefined ? "GET" : "POST",
      headers,
      redirect: "manual",
      ...(data === undefined ? {} : { body: new URLSearchParams(data).toString() }),
    });
    for (const raw of res.headers.getSetCookie()) {
      const [pair = "", ...attrs] = raw.split(";").map((p) => p.trim());
      const i = pair.indexOf("=");
      const name = pair.slice(0, i);
      const value = pair.slice(i + 1);
      const expired = attrs.some((a) => /^max-age=(0|-1)$/i.test(a));
      if (expired || value === "") {
        this.#cookies.delete(name);
      } else {
        this.#cookies.set(name, value);
      }
    }
    return { status: res.status, location: res.headers.get("location"), body: await res.text() };
  }

  /** Mengikuti redirect selama masih di dalam Keycloak; berhenti di halaman atau redirect ke klien. */
  async follow(page: Page): Promise<Page> {
    let p = page;
    while (p.location?.startsWith(this.#base) === true) {
      p = await this.open(p.location);
    }
    return p;
  }

  async submit(page: Page, formId: string, values: Record<string, string>): Promise<Page> {
    const f = form(page.body, formId);
    if (f === null) {
      throw new Error(`formulir ${formId} tidak ditemukan (HTTP ${String(page.status)})`);
    }
    await new Promise((r) => setTimeout(r, JEDA_MS));
    return this.follow(await this.open(f.action, { ...f.inputs, ...values }));
  }
}

/**
 * Login lengkap di halaman Keycloak dari URL otorisasi sampai redirect ke klien. Login pertama mendaftarkan
 * TOTP lalu menyimpan kode pemulihan; login berikutnya memasukkan OTP. Mengembalikan query string callback.
 */
export async function loginThroughKeycloak(kc: EphemeralKeycloak, authUrl: URL, user: TestUser): Promise<string> {
  const b = new KcBrowser(kc.baseUrl);
  let p = await b.follow(await b.open(authUrl.href));
  p = await b.submit(p, "kc-form-login", { username: user.email, password: user.password, credentialId: "" });
  if (form(p.body, "kc-totp-settings-form") !== null) {
    user.otpKey = form(p.body, "kc-totp-settings-form")?.inputs["totpSecret"] ?? null;
    p = await b.submit(p, "kc-totp-settings-form", { totp: await user.otp(), userLabel: "uji" });
    p = await b.submit(p, "kc-recovery-codes-settings-form", { kcRecoveryCodesConfirmationCheck: "on" });
  } else {
    p = await b.submit(p, "kc-otp-login-form", { otp: await user.otp() });
  }
  if (p.location?.startsWith(BACKOFFICE_REDIRECT) !== true) {
    throw new Error(`login tidak berakhir di redirect klien (HTTP ${String(p.status)})`);
  }
  return new URL(p.location).search;
}
