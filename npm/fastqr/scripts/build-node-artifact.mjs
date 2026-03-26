import { cpSync, existsSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(scriptDir, "../../..");
const packageDir = resolve(rootDir, "npm/fastqr");
const nativeDir = resolve(packageDir, "native");

const cargo = spawnSync("cargo", ["build", "-p", "fastqr-napi", "--release"], {
  cwd: rootDir,
  stdio: "inherit",
  shell: process.platform === "win32",
});

if (cargo.status !== 0) {
  process.exit(cargo.status ?? 1);
}

const sourceFile =
  process.platform === "darwin"
    ? resolve(rootDir, "target/release/libfastqr_napi.dylib")
    : process.platform === "win32"
      ? resolve(rootDir, "target/release/fastqr_napi.dll")
      : resolve(rootDir, "target/release/libfastqr_napi.so");

if (!existsSync(sourceFile)) {
  throw new Error(`native artifact was not produced: ${sourceFile}`);
}

mkdirSync(nativeDir, { recursive: true });
cpSync(sourceFile, resolve(nativeDir, "fastqr-napi.node"));
