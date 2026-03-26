import { existsSync, mkdirSync, rmSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(scriptDir, "../../..");
const crateDir = resolve(rootDir, "crates/fastqr_wasm");
const outDir = resolve(rootDir, "npm/fastqr/wasm");
const crateOutDir = resolve(crateDir, "npm/fastqr/wasm");
const toolsRoot = resolve(rootDir, "target/tools");
const wasmPackVersion = "0.13.1";
const wasmPackCommand =
  process.platform === "win32"
    ? resolve(toolsRoot, "bin/wasm-pack.exe")
    : resolve(toolsRoot, "bin/wasm-pack");

function run(command, args, options) {
  const child = spawnSync(command, args, options);
  if (child.error) {
    throw new Error(`failed to start ${command}: ${child.error.message}`);
  }
  if (child.status !== 0) {
    process.exit(child.status ?? 1);
  }
  return child;
}

function ensureWasmPack() {
  if (existsSync(wasmPackCommand)) {
    const version = spawnSync(wasmPackCommand, ["--version"], {
      encoding: "utf8",
      shell: process.platform === "win32",
    });
    if (version.status === 0 && version.stdout.includes(wasmPackVersion)) {
      return;
    }
  }

  mkdirSync(toolsRoot, { recursive: true });
  run(
    "cargo",
    ["install", "--locked", "wasm-pack", "--version", wasmPackVersion, "--root", toolsRoot],
    {
      cwd: rootDir,
      stdio: "inherit",
      shell: process.platform === "win32",
    },
  );
}

rmSync(outDir, { recursive: true, force: true });
rmSync(crateOutDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

ensureWasmPack();
run(
  wasmPackCommand,
  [
    "build",
    ".",
    "--target",
    "web",
    "--out-dir",
    "../../npm/fastqr/wasm",
    "--out-name",
    "fastqr_wasm",
  ],
  {
    cwd: crateDir,
    stdio: "inherit",
    shell: process.platform === "win32",
  },
);
