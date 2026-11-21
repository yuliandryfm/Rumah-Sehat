import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// Satu konfigurasi untuk seluruh workspace; `projects` memisahkan lingkungan.
// Tanpa `passWithNoTests`: berkas test yang hilang harus menggagalkan, bukan lulus diam-diam.
export default defineConfig({
  plugins: [react()],
  test: {
    projects: [
      {
        extends: true,
        test: {
          name: "node",
          environment: "node",
          include: [
            "*/src/**/*.test.ts",
            "packages/api-client/src/**/*.test.ts",
            "packages/ui/scripts/**/*.test.ts",
            "packages/bff-auth/src/**/*.test.ts",
          ],
          exclude: ["**/node_modules/**", "**/*.integration.test.ts"],
        },
      },
      {
        // Test integrasi BFF (F1-T41/F1-T42): Valkey, OpenBao, dan Keycloak sungguhan di kontainer sekali
        // pakai (docker wajib; tanpa docker test GAGAL, tidak dilewati). Image dari infra/local/compose.yaml.
        extends: true,
        test: {
          name: "integrasi",
          environment: "node",
          include: ["packages/bff-auth/src/**/*.integration.test.ts"],
          testTimeout: 120_000,
          hookTimeout: 420_000,
        },
      },
      {
        extends: true,
        test: {
          name: "ui",
          environment: "jsdom",
          // jsdom + axe-core di mesin pengembangan bersama yang sibuk (load 15-30) bisa melewati batas 5 detik
          // bawaan (gagal palsu 2026-11-21); asersi tidak berubah.
          testTimeout: 30_000,
          include: ["packages/ui/src/**/*.test.{ts,tsx}", "packages/bff-auth/src/client/**/*.test.tsx", "situs/src/**/*.test.tsx", "backoffice/src/**/*.test.tsx"],
        },
      },
    ],
  },
});
