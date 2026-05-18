# fastqr npm Package

This directory contains the JavaScript-facing package entrypoints for `fastqr`.

- `node.js`: ESM loader for the compiled N-API addon from `./native`
- `browser.js`: browser/WASM wrapper around the generated `wasm-bindgen` package from `./wasm`

The published package is ESM only.

## Node usage

```js
import { encodeText, renderPng, decodeImage } from "fastqr";

const code = encodeText("https://example.com", {
  minErrorCorrection: "Q",
  minVersion: 2,
  maxVersion: 10,
  mask: 3,
  boostErrorCorrection: false,
});

const png = renderPng("https://example.com", {
  errorCorrection: "H",
  scale: 6,
  border: 2,
  dark: "#111827",
  light: "#ffffff",
});

const decoded = decodeImage(png, { tryInvert: true });
console.log(code.version, decoded.text);
```

The legacy positional error correction call remains supported:

```js
encodeText("FASTQR", "H");
```

## Browser usage

```js
import init, { encodeText, decodeRgba } from "fastqr/browser";

await init();

const code = encodeText("FASTQR", {
  minErrorCorrection: "H",
  minVersion: 2,
  mask: 1,
});
const rgba = code.renderRgba({
  scale: 8,
  border: 4,
  dark: "#111827",
  light: "#ffffff",
});
const side = (code.size + 4 * 2) * 8;
const decoded = decodeRgba(rgba, side, side, { tryInvert: false });
console.log(code.mask, decoded.text);
```

Build outputs are generated into `./native` and `./wasm` through Vite+ tasks:

```sh
vp run build-node
vp run build-browser
vp run build
```

Native addon builds are stored as platform-tagged prebuilds under
`native/<platform>-<arch>/fastqr-napi.node`. The runtime loader selects the
current platform tag and fails with an install/build message when that prebuild
is missing.

License: `GPL-3.0-only`.
