import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageDir = resolve(scriptDir, "..");
const rootDir = resolve(packageDir, "../..");
const vuePackageDir = resolve(rootDir, "npm/fastqr_vue");
const platformTag = `${process.platform}-${process.arch}`;
const requiredNodeArtifacts = ["node.d.ts", "node.js", `native/${platformTag}/fastqr-napi.node`];
const requiredBrowserArtifacts = [
  "browser.d.ts",
  "browser.js",
  "wasm/fastqr_wasm.d.ts",
  "wasm/fastqr_wasm.js",
  "wasm/fastqr_wasm_bg.wasm",
  "wasm/fastqr_wasm_bg.wasm.d.ts",
  "wasm/package.json",
];
const requiredVueArtifacts = ["README.md", "dist/index.d.ts", "dist/index.js", "package.json"];
const shouldCheckNode = process.argv.includes("--node");
const shouldCheckBrowser = process.argv.includes("--browser");
const shouldCheckVue = process.argv.includes("--vue");

if (!shouldCheckNode && !shouldCheckBrowser && !shouldCheckVue) {
  throw new Error("expected at least one artifact group to check");
}

const tempDir = mkdtempSync(resolve(tmpdir(), "fastqr-pack-"));

try {
  if (shouldCheckNode || shouldCheckBrowser) {
    await smokeFastqrPackage(tempDir);
  }
  if (shouldCheckVue) {
    smokeVuePackage(tempDir);
  }
} finally {
  rmSync(tempDir, { recursive: true, force: true });
}

async function smokeFastqrPackage(tempDir) {
  const { extractedPackageDir, packedFiles } = packAndExtract(packageDir, tempDir, "fastqr");
  const missing = [];

  if (shouldCheckNode) {
    missing.push(...missingFiles(requiredNodeArtifacts, packedFiles));
    if (packedFiles.has("native/fastqr-napi.node")) {
      throw new Error("npm package must not include an untagged native/fastqr-napi.node");
    }
  }
  if (shouldCheckBrowser) {
    missing.push(...missingFiles(requiredBrowserArtifacts, packedFiles));
  }
  if (missing.length > 0) {
    throw new Error(`fastqr package is missing required artifacts: ${missing.join(", ")}`);
  }

  if (shouldCheckNode) {
    await smokeNodeRuntime(extractedPackageDir);
    smokeNodeTypes(extractedPackageDir);
  }
  if (shouldCheckBrowser) {
    await smokeBrowserRuntime(extractedPackageDir);
    smokeBrowserTypes(extractedPackageDir);
  }
}

function smokeVuePackage(tempDir) {
  const { extractedPackageDir, packedFiles } = packAndExtract(vuePackageDir, tempDir, "fastqr-vue");
  const missing = missingFiles(requiredVueArtifacts, packedFiles);
  if (missing.length > 0) {
    throw new Error(`@fastqr/vue package is missing required artifacts: ${missing.join(", ")}`);
  }

  const packageJson = JSON.parse(
    readFileSync(resolve(extractedPackageDir, "package.json"), "utf8"),
  );
  if (packageJson.types !== "./dist/index.d.ts") {
    throw new Error("@fastqr/vue package types must point at ./dist/index.d.ts");
  }
  if (packageJson.exports?.["."]?.types !== "./dist/index.d.ts") {
    throw new Error("@fastqr/vue export types must point at ./dist/index.d.ts");
  }
}

function packAndExtract(cwd, tempDir, label) {
  const packDir = mkdtempSync(resolve(tempDir, `${label}-`));
  const pack = run("npm", ["pack", "--json", "--pack-destination", packDir], {
    cwd,
    encoding: "utf8",
  });
  const [summary] = JSON.parse(pack.stdout);
  const packedFiles = new Set(summary.files.map((file) => file.path));
  const tarball = resolve(packDir, summary.filename);
  run("tar", ["-xzf", tarball, "-C", packDir]);
  return {
    extractedPackageDir: resolve(packDir, "package"),
    packedFiles,
  };
}

function missingFiles(paths, packedFiles) {
  return paths.filter((path) => !packedFiles.has(path));
}

async function smokeNodeRuntime(extractedPackageDir) {
  const module = await import(pathToFileURL(resolve(extractedPackageDir, "node.js")));
  for (const name of [
    "decodeImage",
    "decodeRgba",
    "encodeText",
    "renderJpeg",
    "renderPng",
    "renderWebp",
  ]) {
    if (typeof module[name] !== "function") {
      throw new Error(`node runtime export is not a function: ${name}`);
    }
    if (module.default?.[name] !== module[name]) {
      throw new Error(`node default export is missing function: ${name}`);
    }
  }
  if (typeof module.default !== "object" || module.default === null) {
    throw new Error("node runtime default export is not the API object");
  }
  const code = module.encodeText("fastqr", {
    boostErrorCorrection: false,
    mask: 1,
    minErrorCorrection: "H",
    minVersion: 2,
    maxVersion: 10,
  });
  if (code.version < 2 || code.mask !== 1 || code.errorCorrection !== "H") {
    throw new Error("node encodeText did not honor encode options");
  }
  if (code.size < 25 || !(code.modules instanceof Uint8Array)) {
    throw new Error("node encodeText returned an invalid QR object");
  }
  const png = module.renderPng("fastqr", {
    border: 1,
    dark: "#111827",
    errorCorrection: "H",
    light: "#ffffff",
    scale: 2,
  });
  if (!(png instanceof Uint8Array) || png.length === 0) {
    throw new Error("node renderPng returned an invalid image buffer");
  }
  const decoded = module.decodeImage(png, { maxPixels: 512 * 512, tryInvert: true });
  if (decoded.text !== "fastqr") {
    throw new Error("node decodeImage did not roundtrip the rendered PNG");
  }
}

async function smokeBrowserRuntime(extractedPackageDir) {
  const module = await import(pathToFileURL(resolve(extractedPackageDir, "browser.js")));
  const expectedExports = [
    "decodeCanvas",
    "decodeRgba",
    "decodeVideoFrame",
    "default",
    "encodeText",
    "renderToCanvas",
  ];
  const actualExports = Object.keys(module).sort();
  const unexpected = actualExports.filter((name) => !expectedExports.includes(name));
  const missing = expectedExports.filter((name) => !actualExports.includes(name));
  if (missing.length > 0 || unexpected.length > 0) {
    throw new Error(
      `browser runtime exports changed; missing: ${missing.join(", ") || "none"}; unexpected: ${
        unexpected.join(", ") || "none"
      }`,
    );
  }
  for (const name of expectedExports.filter((name) => name !== "default")) {
    if (typeof module[name] !== "function") {
      throw new Error(`browser runtime export is not a function: ${name}`);
    }
  }
  if (typeof module.default !== "function") {
    throw new Error("browser runtime default export is not the WASM initializer");
  }
}

function smokeNodeTypes(extractedPackageDir) {
  smokeTypes(
    extractedPackageDir,
    "node-type-smoke.ts",
    `
      import fastqr, {
        decodeImage,
        decodeRgba,
        encodeText,
        renderJpeg,
        renderPng,
        renderWebp,
        type DecodedQr,
        type EncodeOptions,
        type QrCode,
        type RenderOptions,
      } from "fastqr";
      import { encodeText as encodeTextFromNode } from "fastqr/node";

      const encodeOptions: EncodeOptions = { errorCorrection: "Q", minVersion: 1, maxVersion: 10 };
      const renderOptions: RenderOptions = { scale: 2, border: 1, dark: "#000000", light: "#ffffff" };
      const qr: QrCode = encodeText("fastqr", encodeOptions);
      const qrFromNode: QrCode = encodeTextFromNode("fastqr", "H");
      const png: Uint8Array = renderPng("fastqr", renderOptions);
      const jpeg: Uint8Array = renderJpeg("fastqr", renderOptions);
      const webp: Uint8Array = renderWebp("fastqr", renderOptions);
      const decodedImage: DecodedQr = decodeImage(png, { tryInvert: true });
      const decodedRgba: DecodedQr = decodeRgba(new Uint8Array(21 * 21 * 4), 21, 21, { tryInvert: false });
      fastqr.encodeText("fastqr", encodeOptions);
      qrFromNode.modules.byteLength + jpeg.byteLength + webp.byteLength + decodedImage.bytes.byteLength + decodedRgba.bytes.byteLength;
    `,
  );
}

function smokeBrowserTypes(extractedPackageDir) {
  smokeTypes(
    extractedPackageDir,
    "browser-type-smoke.ts",
    `
      import init, {
        decodeCanvas,
        decodeRgba,
        decodeVideoFrame,
        encodeText,
        renderToCanvas,
        type BrowserDecodedQr,
        type BrowserEncodeOptions,
        type BrowserQrCode,
        type BrowserRenderOptions,
      } from "fastqr/browser";

      const encodeOptions: BrowserEncodeOptions = { errorCorrection: "H", minVersion: 1, maxVersion: 20 };
      const renderOptions: BrowserRenderOptions = { scale: 2, border: 1, dark: "#111827", light: "#ffffff" };
      const ready: Promise<unknown> = init();
      const qr: BrowserQrCode = encodeText("fastqr", encodeOptions);
      const decoded: BrowserDecodedQr = decodeRgba(new Uint8Array(21 * 21 * 4), 21, 21, { tryInvert: true });
      declare const canvas: HTMLCanvasElement;
      declare const video: HTMLVideoElement;
      renderToCanvas(canvas, "fastqr", renderOptions);
      decodeCanvas(canvas, { tryInvert: false });
      decodeVideoFrame(video, canvas, { tryInvert: true });
      ready.then(() => qr.modules().byteLength + decoded.bytes.byteLength);
    `,
  );
}

function smokeTypes(extractedPackageDir, filename, source) {
  const smokeDir = resolve(extractedPackageDir, ".smoke");
  const sourceFile = resolve(smokeDir, filename);
  const tsconfigFile = resolve(smokeDir, "tsconfig.json");
  const typescriptBin =
    process.platform === "win32"
      ? resolve(rootDir, "node_modules/.bin/tsc.cmd")
      : resolve(rootDir, "node_modules/.bin/tsc");

  mkdirSync(smokeDir, { recursive: true });
  writeFileSync(sourceFile, source);
  writeFileSync(
    tsconfigFile,
    JSON.stringify(
      {
        compilerOptions: {
          allowSyntheticDefaultImports: true,
          baseUrl: ".",
          lib: ["DOM", "ES2022"],
          module: "ESNext",
          moduleResolution: "Bundler",
          noEmit: true,
          paths: {
            fastqr: [resolve(extractedPackageDir, "node.d.ts")],
            "fastqr/browser": [resolve(extractedPackageDir, "browser.d.ts")],
            "fastqr/node": [resolve(extractedPackageDir, "node.d.ts")],
          },
          strict: true,
          target: "ES2022",
          verbatimModuleSyntax: true,
        },
        include: [sourceFile],
      },
      null,
      2,
    ),
  );

  run(typescriptBin, ["-p", tsconfigFile], { cwd: extractedPackageDir, stdio: "inherit" });
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    shell: process.platform === "win32",
    ...options,
  });
  if (result.error) {
    throw new Error(`failed to start ${command}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    if (result.stderr) {
      process.stderr.write(result.stderr);
    }
    process.exit(result.status ?? 1);
  }
  return result;
}
