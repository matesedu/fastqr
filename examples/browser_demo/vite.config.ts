import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { defineConfig } from "vite-plus";

const packageDir = dirname(fileURLToPath(import.meta.url));
const workspaceDir = resolve(packageDir, "../..");

const appInputs = [
  "index.html",
  "package.json",
  "src/**",
  "tsconfig.json",
  "vite.config.ts",
  "!dist/**",
];

export default defineConfig({
  fmt: {
    ignorePatterns: ["dist/**", "node_modules/**"],
    semi: true,
    singleQuote: false,
    trailingComma: "all",
  },
  lint: {
    ignorePatterns: ["dist/**", "node_modules/**"],
  },
  optimizeDeps: {
    exclude: ["fastqr"],
  },
  preview: {
    host: "0.0.0.0",
    port: 4173,
    strictPort: true,
  },
  server: {
    fs: {
      allow: [workspaceDir],
    },
    host: "0.0.0.0",
    port: 4173,
    strictPort: true,
  },
  run: {
    tasks: {
      check: {
        command: "vp check",
        input: appInputs,
      },
      fmt: {
        command: "vp check --fix",
        input: appInputs,
      },
      dev: {
        command: "vp dev --host 0.0.0.0 --port 4173 --strictPort",
        cache: false,
        dependsOn: ["fastqr#build-browser"],
      },
      build: {
        command: "vp build",
        dependsOn: ["fastqr#build-browser"],
        env: ["NODE_ENV", "VITE_*"],
        input: appInputs,
      },
      preview: {
        command: "vp preview --host 0.0.0.0 --port 4173 --strictPort",
        cache: false,
        dependsOn: ["build"],
      },
    },
  },
});
