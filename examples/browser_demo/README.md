# Browser Demo

Run locally through the root task:

```sh
vp run demo
```

The demo serves on `http://localhost:4173` by default.

## Camera Behavior

The camera scanner asks the browser for media-device permission and prefers the rear-facing camera when available. Camera frames are drawn to a scratch canvas and decoded locally in WASM; this demo does not upload frames or decoded payloads.

Camera access requires a secure context. Use localhost for development and HTTPS for deployed demos.
