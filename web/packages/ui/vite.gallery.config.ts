import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const paket = dirname(fileURLToPath(import.meta.url));

// Membangun halaman galeri statis ke dist-gallery/ (tanpa CDN atau font jarak jauh).
export default defineConfig({
  root: resolve(paket, "gallery"),
  plugins: [react()],
  build: {
    outDir: resolve(paket, "dist-gallery"),
    emptyOutDir: true,
  },
});
