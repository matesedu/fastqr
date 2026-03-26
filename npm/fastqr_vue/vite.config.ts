import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { defineConfig } from "vite-plus";

const packageDir = dirname(fileURLToPath(import.meta.url));

const packageInputs = [
  "README.md",
  "package.json",
  "src/**",
  "tsconfig.json",
  "vite.config.ts",
  "!dist/**",
];

export default defineConfig({
  build: {
    emptyOutDir: true,
    lib: {
      entry: resolve(packageDir, "src/index.ts"),
      fileName: "index",
      formats: ["es"],
    },
    rollupOptions: {
      external: ["vue", "fastqr", "fastqr/browser"],
    },
  },
  fmt: {
    ignorePatterns: ["dist/**", "node_modules/**"],
    semi: true,
    singleQuote: false,
    trailingComma: "all",
  },
  lint: {
    ignorePatterns: ["dist/**", "node_modules/**"],
  },
  run: {
    tasks: {
      check: {
        command: "vp check",
        input: packageInputs,
      },
      fmt: {
        command: "vp check --fix",
        input: packageInputs,
      },
      build: {
        command: "vp build",
        dependsOn: ["check"],
        input: packageInputs,
      },
    },
  },
});
