import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite-plus";

const packageDir = dirname(fileURLToPath(import.meta.url));
const workspaceDir = resolve(packageDir, "..");

const appInputs = [
  "index.html",
  "package.json",
  "src/**",
  "tsconfig.json",
  "vite.config.ts",
  "!dist/**",
];

export default defineConfig({
  base: process.env.PAGES_BASE_PATH ?? "/",
  define: {
    __VUE_OPTIONS_API__: false,
    __VUE_PROD_DEVTOOLS__: false,
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: false,
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
  optimizeDeps: {
    exclude: ["fastqr", "@fastqr/vue"],
  },
  plugins: [vue()],
  preview: {
    host: "0.0.0.0",
    port: 4174,
    strictPort: true,
  },
  server: {
    fs: {
      allow: [workspaceDir],
    },
    host: "0.0.0.0",
    port: 4174,
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
        command: "vp dev --host 0.0.0.0 --port 4174 --strictPort",
        cache: false,
        dependsOn: ["fastqr#build-browser", "@fastqr/vue#build"],
      },
      build: {
        command: "vp build",
        dependsOn: ["fastqr#build-browser", "@fastqr/vue#build"],
        env: ["NODE_ENV", "VITE_*", "PAGES_BASE_PATH"],
        input: appInputs,
      },
      preview: {
        command: "vp preview --host 0.0.0.0 --port 4174 --strictPort",
        cache: false,
        dependsOn: ["build"],
      },
    },
  },
});
