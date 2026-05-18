# Shell Examples

- `quick_start.sh`: render, decode, and inspect a single QR code
- `batch_render.sh`: render multiple payloads into a folder

All commands are expressed through `vp run`.

The scripts write to `./fastqr-shell-example.png` and `./fastqr-batch` by default, or to the path you pass as an argument. Existing files with the same names are overwritten.

QR payloads may contain secrets. Do not commit generated images or sample payloads that include real Wi-Fi passwords, tokens, or personal data.
