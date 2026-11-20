// ESLint flat config untuk seluruh workspace web (stack §1.2; coding-standards §2.3).
// Aturan type-aware (strictTypeChecked) memakai typescript 6.0.3 sebagai API parser; kompilasi
// tipe resmi memakai typescript 7.0.2 (lihat web/package.json dan docs/sources.md SRC-500 s.d. SRC-509).
import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: ["**/node_modules/**", "**/.next/**", "**/next-env.d.ts", "**/dist/**", "**/dist-gallery/**", "**/coverage/**", "packages/api-client/src/generated/**"],
  },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      "@typescript-eslint/consistent-type-imports": "error",
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-non-null-assertion": "error",
      "no-console": "error",
      eqeqeq: ["error", "always"],
    },
  },
  {
    // Skrip Node (.mjs) dan konfigurasi: tanpa tipe, boleh console.
    files: ["**/*.mjs", "eslint.config.js"],
    extends: [tseslint.configs.disableTypeChecked],
    languageOptions: { globals: { process: "readonly", console: "readonly", fetch: "readonly", setTimeout: "readonly" } },
    rules: { "no-console": "off" },
  },
);
