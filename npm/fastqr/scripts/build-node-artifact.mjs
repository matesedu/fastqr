import { cpSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(scriptDir, "../../..");
const packageDir = resolve(rootDir, "npm/fastqr");
const nativeDir = resolve(packageDir, "native");
const manifestPath = resolve(rootDir, "crates/fastqr_napi/Cargo.toml");
const platformTag = `${process.platform}-${process.arch}`;

const cargo = spawnSync("cargo", ["build", "--manifest-path", manifestPath, "--release"], {
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

rmSync(nativeDir, { recursive: true, force: true });
mkdirSync(resolve(nativeDir, platformTag), { recursive: true });
cpSync(sourceFile, resolve(nativeDir, platformTag, "fastqr-napi.node"));
