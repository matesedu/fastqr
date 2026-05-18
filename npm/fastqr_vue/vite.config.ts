import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { defineConfig } from "vite-plus";

const packageDir = dirname(fileURLToPath(import.meta.url));

const packageInputs = [
  "README.md",
  "package.json",
  "scripts/**/*.mjs",
  "src/**",
  "tsconfig.build.json",
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
        command: "vp check && tsc --noEmit --pretty false",
        input: packageInputs,
      },
      fmt: {
        command: "vp check --fix",
        input: packageInputs,
      },
      build: {
        command: "vp build && tsc --project tsconfig.build.json --pretty false",
        dependsOn: ["check"],
        input: packageInputs,
      },
      "pack-smoke": {
        command: "node ./scripts/check-package-artifacts.mjs",
        dependsOn: ["build"],
        input: packageInputs,
      },
    },
  },
});
