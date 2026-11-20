// Kontainer sekali pakai untuk test integrasi BFF (KHUSUS TEST). Pola sama dengan scripts/keycloak-check.sh
// mode ephemeral: image dipin satu kali di infra/local/compose.yaml (tag@digest), port acak di 127.0.0.1,
// nilai rahasia diteruskan lewat `-e NAMA` dari lingkungan proses (tidak muncul di argumen), kontainer
// selalu dihapus. Tanpa docker, test GAGAL dengan pesan jelas; tidak dilewati diam-diam.

import { randomBytes } from "node:crypto";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

/** Akar repositori (web/packages/bff-auth/src/testing → lima tingkat ke atas). */
export const REPO_ROOT = fileURLToPath(new URL("../../../../../", import.meta.url));

export function composeImage(repo: string): string {
  const compose = readFileSync(`${REPO_ROOT}infra/local/compose.yaml`, "utf8");
  const esc = repo.replace(/[.*+?^${}()|[\]\\/]/g, "\\$&");
  const m = new RegExp(`^\\s*image:\\s*(${esc}:[^\\s@]+@sha256:[0-9a-f]{64})\\s*$`, "m").exec(compose);
  if (m?.[1] === undefined) {
    throw new Error(`image ${repo} (tag@digest) tidak ditemukan di infra/local/compose.yaml`);
  }
  return m[1];
}

export interface DockerResult {
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
}

export function docker(args: readonly string[], env: Record<string, string> = {}, timeoutMs = 600_000): DockerResult {
  const r = spawnSync("docker", args, {
    env: { ...process.env, ...env },
    encoding: "utf8",
    timeout: timeoutMs,
    maxBuffer: 32 * 1024 * 1024,
  });
  if (r.error !== undefined) {
    throw new Error(`docker tidak dapat dijalankan (${r.error.message}); test integrasi BFF butuh docker dan GAGAL tanpa docker`);
  }
  return { status: r.status ?? -1, stdout: r.stdout, stderr: r.stderr };
}

export function dockerOk(args: readonly string[], env: Record<string, string> = {}, timeoutMs?: number): string {
  const r = docker(args, env, timeoutMs);
  if (r.status !== 0) {
    throw new Error(`docker ${args.slice(0, 2).join(" ")} gagal (status ${String(r.status)}): ${r.stderr.slice(-800)}`);
  }
  return r.stdout.trim();
}

export function randomSecret(): string {
  return randomBytes(24).toString("base64url");
}

export class Container {
  readonly id: string;

  private constructor(id: string) {
    this.id = id;
  }

  /**
   * Menjalankan `docker run -d`. `env` berisi nilai rahasia yang diteruskan lewat `-e NAMA` saja;
   * `plainEnv` untuk nilai non-rahasia.
   */
  static run(options: {
    readonly image: string;
    readonly ports?: readonly number[];
    readonly env?: Record<string, string>;
    readonly plainEnv?: Record<string, string>;
    readonly extra?: readonly string[];
    readonly cmd?: readonly string[];
  }): Container {
    const name = `rs-bff-uji-${String(process.pid)}-${randomBytes(4).toString("hex")}`;
    const args = ["run", "-d", "--name", name];
    for (const p of options.ports ?? []) {
      args.push("-p", `127.0.0.1::${String(p)}`);
    }
    for (const k of Object.keys(options.env ?? {})) {
      args.push("-e", k);
    }
    for (const [k, v] of Object.entries(options.plainEnv ?? {})) {
      args.push("-e", `${k}=${v}`);
    }
    args.push(...(options.extra ?? []), options.image, ...(options.cmd ?? []));
    dockerOk(args, options.env ?? {});
    return new Container(name);
  }

  hostPort(port: number): number {
    const out = dockerOk(["port", this.id, `${String(port)}/tcp`]);
    const m = /^127\.0\.0\.1:(\d+)$/m.exec(out);
    if (m?.[1] === undefined) {
      throw new Error(`port ${String(port)} kontainer tidak dipetakan ke 127.0.0.1`);
    }
    return Number(m[1]);
  }

  logs(): string {
    const r = docker(["logs", "--tail", "40", this.id]);
    return `${r.stdout}${r.stderr}`;
  }

  remove(): void {
    docker(["rm", "-f", this.id]);
  }
}

export async function waitFor(check: () => Promise<boolean>, timeoutMs: number, label: string): Promise<void> {
  const until = Date.now() + timeoutMs;
  while (Date.now() < until) {
    try {
      if (await check()) {
        return;
      }
    } catch {
      // belum siap
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`${label} tidak siap dalam ${String(timeoutMs / 1000)} detik`);
}
