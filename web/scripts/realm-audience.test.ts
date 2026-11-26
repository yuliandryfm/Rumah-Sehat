// CR-028: realm-as-code harus memuat audience mapper untuk klien BFF, kalau tidak API backend menolak
// token BFF dengan `wrong_audience`. Nilai audiens mengikuti AUTH_ADMIN_AUDIENCE / AUTH_CORPORATE_AUDIENCE
// di konfigurasi rs-backend (apps/rs-backend/src/config.rs). Pemeriksaan hidup terhadap Keycloak ada di
// infra/keycloak/test/keycloak_check.py.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const realms = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "infra", "keycloak", "realms");

interface Mapper {
  name: string;
  protocolMapper: string;
  config: Record<string, string>;
}
interface Klien {
  clientId: string;
  protocolMappers?: Mapper[];
}

const KASUS = [
  { realm: "korporat", klien: "rs-korporat-bff", audiens: "rs-corporate-api" },
  { realm: "admin", klien: "rs-backoffice-bff", audiens: "rs-backoffice-api" },
] as const;

describe("audience mapper klien BFF di realm-as-code (CR-028)", () => {
  for (const { realm, klien, audiens } of KASUS) {
    it(`${klien}: satu oidc-audience-mapper ke ${audiens}, hanya access token`, () => {
      const json = JSON.parse(readFileSync(join(realms, `${realm}.json`), "utf8")) as { clients: Klien[] };
      const k = json.clients.find((c) => c.clientId === klien);
      expect(k).toBeDefined();
      const pm = k?.protocolMappers ?? [];
      expect(pm).toHaveLength(1);
      expect(pm[0]?.protocolMapper).toBe("oidc-audience-mapper");
      expect(pm[0]?.config["included.custom.audience"]).toBe(audiens);
      expect(pm[0]?.config["access.token.claim"]).toBe("true");
      expect(pm[0]?.config["id.token.claim"]).toBe("false");
    });
  }
});
