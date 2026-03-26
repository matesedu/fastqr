import { defineConfig } from "vite-plus";

const rustInputs = ["Cargo.toml", "Cargo.lock", "crates/**", "!target/**"];

const rootWebInputs = [
  ".github/**",
  ".gitignore",
  "package.json",
  "pnpm-workspace.yaml",
  "README.md",
  "vite.config.ts",
  "!examples/**",
  "!npm/**",
  "!playground/**",
];

export default defineConfig({
  fmt: {
    ignorePatterns: ["**/dist/**", "**/node_modules/**", "**/target/**"],
    semi: true,
    singleQuote: false,
    trailingComma: "all",
  },
  lint: {
    ignorePatterns: ["**/dist/**", "**/node_modules/**", "**/target/**"],
  },
  run: {
    cache: {
      scripts: false,
      tasks: true,
    },
    tasks: {
      setup: {
        command: "vp install",
        cache: false,
      },
      "config:check": {
        command:
          "vp check .github/workflows/ci.yml .github/workflows/playground-pages.yml .gitignore package.json pnpm-workspace.yaml README.md vite.config.ts",
        input: rootWebInputs,
      },
      "config:fmt": {
        command:
          "vp check --fix .github/workflows/ci.yml .github/workflows/playground-pages.yml .gitignore package.json pnpm-workspace.yaml README.md vite.config.ts",
        input: rootWebInputs,
      },
      "rust:fmt": {
        command: "cargo fmt --all",
        env: ["RUSTFLAGS"],
        input: rustInputs,
      },
      "web:fmt": {
        command:
          "vp run config:fmt && vp run fastqr#fmt && vp run @fastqr/vue#fmt && vp run @fastqr/browser-demo#fmt && vp run @fastqr/example-ts#fmt && vp run @fastqr/example-vue#fmt && vp run @fastqr/playground#fmt",
        env: ["NODE_ENV", "VITE_*"],
        input: [{ auto: true }, "!target/**"],
      },
      "web:check": {
        command:
          "vp run config:check && vp run fastqr#check && vp run @fastqr/vue#check && vp run @fastqr/browser-demo#check && vp run @fastqr/example-ts#check && vp run @fastqr/example-vue#check && vp run @fastqr/playground#check",
        env: ["NODE_ENV", "VITE_*"],
        input: [{ auto: true }, "!target/**"],
      },
      "rust:build": {
        command: "cargo build --workspace",
        env: ["RUSTFLAGS"],
        input: rustInputs,
      },
      "rust:check": {
        command: "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        env: ["RUSTFLAGS"],
        input: rustInputs,
      },
      "rust:test": {
        command: "cargo test --workspace",
        dependsOn: ["rust:check"],
        env: ["RUSTFLAGS"],
        input: rustInputs,
      },
      "example:rust": {
        command: "cargo run --manifest-path examples/rust_basic/Cargo.toml",
        env: ["RUSTFLAGS"],
        input: [...rustInputs, "examples/rust_basic/**"],
      },
      "examples:build": {
        command:
          "vp run @fastqr/browser-demo#build && vp run @fastqr/example-ts#build && vp run @fastqr/example-vue#build",
        env: ["NODE_ENV", "VITE_*"],
        input: [{ auto: true }, "!target/**"],
      },
      "browser:build": {
        command: "vp run fastqr#build-browser && vp run examples:build",
        env: ["NODE_ENV", "VITE_*"],
        input: [{ auto: true }, "!target/**"],
      },
      "playground:build": {
        command: "vp run @fastqr/playground#build",
        env: ["NODE_ENV", "VITE_*", "PAGES_BASE_PATH"],
        input: [{ auto: true }, "!target/**"],
      },
      "demo:dev": {
        command: "vp run @fastqr/browser-demo#dev",
        cache: false,
      },
      "example:ts": {
        command: "vp run @fastqr/example-ts#dev",
        cache: false,
      },
      "example:vue": {
        command: "vp run @fastqr/example-vue#dev",
        cache: false,
      },
      "playground:dev": {
        command: "vp run @fastqr/playground#dev",
        cache: false,
      },
      fmt: {
        command: "vp run rust:fmt && vp run web:fmt",
        env: ["NODE_ENV", "RUSTFLAGS", "VITE_*"],
        input: [{ auto: true }, "!target/**"],
      },
      "workspace:build": {
        command:
          "vp run rust:build && vp run fastqr#build-node && vp run browser:build && vp run playground:build",
        env: ["NODE_ENV", "RUSTFLAGS", "VITE_*", "PAGES_BASE_PATH"],
        input: [{ auto: true }, "!target/**"],
      },
      "workspace:check": {
        command: "vp run web:check && vp run rust:check",
        env: ["NODE_ENV", "RUSTFLAGS", "VITE_*"],
        input: [{ auto: true }, "!target/**"],
      },
      "workspace:test": {
        command: "vp run rust:test",
        env: ["RUSTFLAGS"],
        input: rustInputs,
      },
      "workspace:ci": {
        command: "vp run workspace:check && vp run workspace:test && vp run workspace:build",
        env: ["NODE_ENV", "RUSTFLAGS", "VITE_*"],
        input: [{ auto: true }, "!target/**"],
      },
      build: {
        command: "vp run workspace:build",
        env: ["NODE_ENV", "RUSTFLAGS", "VITE_*", "PAGES_BASE_PATH"],
        input: [{ auto: true }, "!target/**"],
      },
      check: {
        command: "vp run workspace:check",
        env: ["NODE_ENV", "RUSTFLAGS", "VITE_*"],
        input: [{ auto: true }, "!target/**"],
      },
      test: {
        command: "vp run workspace:test",
        env: ["RUSTFLAGS"],
        input: rustInputs,
      },
      ci: {
        command: "vp run workspace:ci",
        env: ["NODE_ENV", "RUSTFLAGS", "VITE_*", "PAGES_BASE_PATH"],
        input: [{ auto: true }, "!target/**"],
      },
      cli: {
        command: "cargo run -p fastqr-tui --bin fastqr --",
        cache: false,
      },
      demo: {
        command: "vp run demo:dev",
        cache: false,
      },
      playground: {
        command: "vp run playground:dev",
        cache: false,
      },
    },
  },
  staged: {
    "*.{js,cjs,mjs,ts,vue,html,css,json,md,yaml,yml}":
      "vp run config:fmt && vp run fastqr#fmt && vp run @fastqr/vue#fmt && vp run @fastqr/browser-demo#fmt && vp run @fastqr/example-ts#fmt && vp run @fastqr/example-vue#fmt && vp run @fastqr/playground#fmt",
    "*.rs": "vp run rust:fmt",
  },
});
