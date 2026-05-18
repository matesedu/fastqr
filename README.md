# fastqr

`fastqr` is a QR Code library with a Rust core, image and TUI support, and thin Node.js, browser, and Vue integrations.

## Install

```sh
pnpm add fastqr
pnpm add @fastqr/vue vue
```

Rust crates are currently developed from this workspace. The published package plan is tracked in the production-readiness issues.

## Node

```js
import { renderPng, decodeImage } from "fastqr";
import { writeFile, readFile } from "node:fs/promises";

await writeFile("code.png", renderPng("https://example.com"));
const decoded = decodeImage(await readFile("code.png"));
console.log(decoded.text ?? decoded.bytes);
```

The native Node package is platform-specific. Do not publish release artifacts until the native distribution model is finalized.

## Browser

```ts
import init, { encodeText } from "fastqr/browser";

await init();
const code = encodeText("https://example.com", "M");
document.querySelector("#svg")!.innerHTML = code.renderSvg(4);
```

Browser decoding and rendering run locally in WASM. Camera scanning requires a secure context: HTTPS in production or localhost during development.

## Vue

```vue
<script setup lang="ts">
import { ref, useTemplateRef } from "vue";
import { useFastqr, useFastqrCanvas } from "@fastqr/vue";

const text = ref("https://example.com");
const canvas = useTemplateRef<HTMLCanvasElement>("canvas");
const fastqr = useFastqr();
const preview = useFastqrCanvas({
  errorCorrection: "M",
  previewCanvas: canvas,
  ready: fastqr.ready,
  text,
});
</script>

<template>
  <textarea v-model="text" />
  <canvas ref="canvas"></canvas>
  <p v-if="preview.renderError">{{ preview.renderError }}</p>
</template>
```

## Commands

```sh
vp install
vp run check
vp run test
vp run build
```

## CLI

```sh
vp run cli -- encode "fastqr"
vp run cli -- render "fastqr" ./code.png
vp run cli -- decode ./code.png
vp run cli -- render "fastqr" ./code.svg --format svg --ecc H
vp run cli -- render "fastqr" ./code.webp --scale 12 --border 4
vp run cli -- encode "fastqr" --invert
```

`render` infers `png`, `jpeg`, `webp`, or `svg` from the output extension unless `--format` is set. Existing output files are overwritten.

## Apps

```sh
vp run demo
vp run playground
vp run example:ts
vp run example:vue
vp run example:rust
```

More examples live in `examples/README.md`.

## Support Matrix

| Surface      | Current support                                                     |
| ------------ | ------------------------------------------------------------------- |
| Rust         | Rust 1.93+ with edition 2024                                        |
| Node.js      | Node 22+ during CI; native addon packaging is not release-ready yet |
| Browser/WASM | Modern browsers with WebAssembly and ES modules                     |
| Vue          | Vue 3.5+                                                            |
| CLI          | Built from the Rust workspace as `fastqr`                           |

## Security

Report vulnerabilities privately through the process in `SECURITY.md`. QR payloads may contain credentials, Wi-Fi secrets, or personal data; treat generated images like the underlying text.

License: `GPL-3.0-only`. See `LICENSE`.
