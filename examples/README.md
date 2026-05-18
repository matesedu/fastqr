# Examples

`examples/` contains runnable and copy-paste-friendly samples across the main surfaces:

- `browser_demo`: fuller browser demo with canvas and media stream
- `sh`: shell recipes built around `vp run cli -- ...`
- `rust_basic`: standalone Rust roundtrip example
- `ts_basic`: minimal TypeScript browser app with `fastqr/browser`
- `vue_basic`: minimal Vue app with `@fastqr/vue`

Browser camera examples request local camera permission and require a secure context: HTTPS in production or localhost during development. Frames are decoded in the browser and are not uploaded by these examples.

Shell examples write image files to the requested output path and overwrite existing files. Generated QR images may expose the full payload when shared or committed.
