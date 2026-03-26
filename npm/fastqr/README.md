# fastqr npm Package

This directory contains the JavaScript-facing package entrypoints for `fastqr`.

- `node.js`: ESM loader for the compiled N-API addon from `./native`
- `browser.js`: re-exports the generated `wasm-bindgen` package from `./wasm`

The published package is ESM only.

Build outputs are generated into `./native` and `./wasm` through Vite+ tasks:

```sh
vp run build-node
vp run build-browser
vp run build
```

License: `GPL-3.0-only`.
