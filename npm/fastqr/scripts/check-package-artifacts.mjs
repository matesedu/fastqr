import { spawnSync } from "node:child_process";

const requiredBrowserArtifacts = [
  "wasm/fastqr_wasm.d.ts",
  "wasm/fastqr_wasm.js",
  "wasm/fastqr_wasm_bg.wasm",
  "wasm/fastqr_wasm_bg.wasm.d.ts",
  "wasm/package.json",
];

const shouldCheckBrowser = process.argv.includes("--browser");

if (!shouldCheckBrowser) {
  throw new Error("expected at least one artifact group to check");
}

const pack = spawnSync("npm", ["pack", "--dry-run", "--json"], {
  encoding: "utf8",
  shell: process.platform === "win32",
});

if (pack.status !== 0) {
  process.stderr.write(pack.stderr);
  process.exit(pack.status ?? 1);
}

const [summary] = JSON.parse(pack.stdout);
const packedFiles = new Set(summary.files.map((file) => file.path));
const missing = [];

if (shouldCheckBrowser) {
  for (const path of requiredBrowserArtifacts) {
    if (!packedFiles.has(path)) {
      missing.push(path);
    }
  }
}

if (missing.length > 0) {
  throw new Error(`npm package is missing required artifacts: ${missing.join(", ")}`);
}
