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
          include: ["*/src/**/*.test.ts", "packages/api-client/src/**/*.test.ts", "packages/ui/scripts/**/*.test.ts"],
        },
      },
      {
        extends: true,
        test: {
          name: "ui",
          environment: "jsdom",
          include: ["packages/ui/src/**/*.test.{ts,tsx}", "situs/src/**/*.test.tsx"],
        },
      },
    ],
  },
});
