// Enkripsi token sesi sebagai JWE terikat ID sesi (ADR-011 butir 7, ADR-024 butir 3, M-06).
//
// - JWE Compact, `alg: "dir"`, `enc: "A256GCM"` lewat jose 6.2.17 (`CompactEncrypt`, `compactDecrypt`
//   dengan daftar izin algoritma) [SRC-590].
// - Kunci konten per sesi = HKDF-SHA256(kunci induk dari OpenBao, salt = "rs-bff/v1/<tujuan>",
//   info = ID sesi mentah). ID sesi mentah hanya ada di cookie browser; Valkey menyimpan hash-nya saja.
//   Akibatnya pembaca Valkey, bahkan yang juga memegang kunci induk, tidak dapat membuka token tanpa
//   cookie, dan ciphertext yang dipindah ke sesi lain gagal dibuka.
// - Header terlindung memuat `kid` (rotasi kunci: kunci sebelumnya hanya untuk membuka) dan `cty`
//   (tujuan), sehingga ciphertext satu tujuan tidak dapat dipakai sebagai tujuan lain.

import { hkdfSync } from "node:crypto";
import { CompactEncrypt, compactDecrypt } from "jose";
import type { BffSecrets, SessionKey } from "./openbao.ts";

export type SealPurpose = "tokens" | "login";

export class TokenCryptoError extends Error {
  override readonly name = "TokenCryptoError";
}

function deriveKey(key: SessionKey, sessionId: string, purpose: SealPurpose): Uint8Array {
  const out = hkdfSync("sha256", key.key.expose(), `rs-bff/v1/${purpose}`, sessionId, 32);
  return new Uint8Array(out);
}

function cty(purpose: SealPurpose): string {
  return `rs-bff-${purpose}+json`;
}

export class TokenSealer {
  readonly #keys: BffSecrets["sessionKeys"];

  constructor(keys: BffSecrets["sessionKeys"]) {
    this.#keys = keys;
  }

  /** Mengenkripsi `payload` (JSON) untuk satu ID sesi dan tujuan. */
  async seal(sessionId: string, purpose: SealPurpose, payload: unknown): Promise<string> {
    if (sessionId === "") {
      throw new TokenCryptoError("ID sesi kosong");
    }
    const current = this.#keys.current;
    const plaintext = new TextEncoder().encode(JSON.stringify(payload));
    return new CompactEncrypt(plaintext)
      .setProtectedHeader({ alg: "dir", enc: "A256GCM", kid: current.id, cty: cty(purpose) })
      .encrypt(deriveKey(current, sessionId, purpose));
  }

  /** Membuka JWE hasil `seal`. Gagal (`TokenCryptoError`) bila ID sesi, tujuan, atau kunci tidak cocok. */
  async open(sessionId: string, purpose: SealPurpose, jwe: string): Promise<unknown> {
    if (sessionId === "") {
      throw new TokenCryptoError("ID sesi kosong");
    }
    try {
      const { plaintext } = await compactDecrypt(
        jwe,
        (header) => {
          if (header.cty !== cty(purpose)) {
            throw new TokenCryptoError("tujuan JWE tidak cocok");
          }
          const key = [this.#keys.current, this.#keys.previous].find((k) => k !== undefined && k.id === header.kid);
          if (key === undefined) {
            throw new TokenCryptoError("kid JWE tidak dikenal");
          }
          return deriveKey(key, sessionId, purpose);
        },
        { keyManagementAlgorithms: ["dir"], contentEncryptionAlgorithms: ["A256GCM"] },
      );
      return JSON.parse(new TextDecoder().decode(plaintext)) as unknown;
    } catch {
      // Sebab rinci tidak diteruskan: tidak membedakan kunci salah, sesi salah, atau isi rusak.
      throw new TokenCryptoError("JWE sesi tidak dapat dibuka");
    }
  }
}
