import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageDir = resolve(scriptDir, "..");
const tempDir = mkdtempSync(resolve(tmpdir(), "fastqr-vue-pack-"));
const requiredFiles = new Set(["README.md", "dist/index.d.ts", "dist/index.js", "package.json"]);

try {
  const pack = spawnSync("npm", ["pack", "--dry-run", "--json"], {
    cwd: packageDir,
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  if (pack.status !== 0) {
    process.stderr.write(pack.stderr);
    process.exit(pack.status ?? 1);
  }
  const [summary] = JSON.parse(pack.stdout);
  const packedFiles = new Set(summary.files.map((file) => file.path));
  const missing = [...requiredFiles].filter((file) => !packedFiles.has(file));
  if (missing.length > 0) {
    throw new Error(`@fastqr/vue package is missing required artifacts: ${missing.join(", ")}`);
  }
} finally {
  rmSync(tempDir, { recursive: true, force: true });
}
