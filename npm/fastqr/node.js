import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const bindingPath = fileURLToPath(new URL("./native/fastqr-napi.node", import.meta.url));

if (!existsSync(bindingPath)) {
  throw new Error(
    "Native addon not found at npm/fastqr/native/fastqr-napi.node. Run `vp run fastqr#build-node` first.",
  );
}

const binding = require(bindingPath);

export const encodeText = binding.encodeText;
export const renderPng = binding.renderPng;
export const renderJpeg = binding.renderJpeg;
export const renderWebp = binding.renderWebp;
export const decodeImage = binding.decodeImage;
export const decodeRgba = binding.decodeRgba;

export default binding;
