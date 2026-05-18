import { defineConfig } from "vite-plus";

const packageInputs = [
  "README.md",
  "browser.d.ts",
  "browser.js",
  "node.d.ts",
  "node.js",
  "package.json",
  "scripts/**/*.mjs",
];

export default defineConfig({
  fmt: {
    ignorePatterns: ["dist/**", "native/**", "node_modules/**", "wasm/**"],
    semi: true,
    singleQuote: false,
    trailingComma: "all",
  },
  lint: {
    ignorePatterns: ["dist/**", "native/**", "node_modules/**", "wasm/**"],
  },
  run: {
    tasks: {
      check: {
        command:
          "vp check README.md browser.d.ts browser.js node.d.ts node.js package.json scripts/**/*.mjs",
        input: packageInputs,
      },
      fmt: {
        command:
          "vp check --fix README.md browser.d.ts browser.js node.d.ts node.js package.json scripts/**/*.mjs",
        input: packageInputs,
      },
      "build-node": {
        command: "node ./scripts/build-node-artifact.mjs",
        dependsOn: ["check"],
        env: ["NODE_ENV", "RUSTFLAGS"],
        input: [
          ...packageInputs,
          "../../Cargo.toml",
          "../../Cargo.lock",
          "../../crates/fastqr_core/**",
          "../../crates/fastqr_image/**",
          "../../crates/fastqr_napi/**",
          "!../../target/**",
          "!./native/**",
        ],
      },
      "build-browser": {
        command: "node ./scripts/build-browser-artifact.mjs",
        dependsOn: ["check"],
        env: ["NODE_ENV", "VITE_*"],
        input: [
          ...packageInputs,
          "../../Cargo.toml",
          "../../Cargo.lock",
          "../../crates/fastqr_core/**",
          "../../crates/fastqr_image/**",
          "../../crates/fastqr_wasm/**",
          "!../../target/**",
          "!./wasm/**",
        ],
      },
      build: {
        command: "vp run build-node && vp run build-browser",
        env: ["NODE_ENV", "RUSTFLAGS", "VITE_*"],
        input: [{ auto: true }, "!./native/**", "!./wasm/**", "!../../target/**"],
      },
      "pack-smoke": {
        command: "node ./scripts/check-package-artifacts.mjs --node --browser",
        dependsOn: ["build"],
        env: ["NODE_ENV"],
        input: [{ auto: true }, "!../../target/**"],
      },
    },
  },
});
