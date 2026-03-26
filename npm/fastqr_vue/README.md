# @fastqr/vue

Vue integration helpers for `fastqr`.

The package ships composables for the common browser-side flows:

- `useFastqr()`: boot the WASM runtime
- `useFastqrCanvas()`: render a QR code to canvas and SVG
- `useFastqrDecode()`: decode uploaded raster files
- `useFastqrCamera()`: scan from a live camera loop

```ts
import { useTemplateRef } from "vue";
import { useFastqr, useFastqrCanvas } from "@fastqr/vue";

const previewCanvas = useTemplateRef<HTMLCanvasElement>("previewCanvas");
const fastqr = useFastqr();
const preview = useFastqrCanvas({
  errorCorrection: "M",
  previewCanvas,
  ready: fastqr.ready,
  text: "https://github.com/mates-inc/fastqr",
});
```

License: `GPL-3.0-only`.
