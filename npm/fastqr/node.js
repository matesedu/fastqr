import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const platformTag = `${process.platform}-${process.arch}`;
const bindingPath = fileURLToPath(
  new URL(`./native/${platformTag}/fastqr-napi.node`, import.meta.url),
);

if (!existsSync(bindingPath)) {
  throw new Error(
    `Native addon for ${platformTag} was not found. This package uses platform-tagged prebuilds; reinstall fastqr for this platform or run \`vp run fastqr#build-node\` before packing.`,
  );
}

const binding = require(bindingPath);

export const encodeText = binding.encodeText;
export const renderPng = binding.renderPng;
export const renderJpeg = binding.renderJpeg;
export const renderWebp = binding.renderWebp;
export const decodeImage = binding.decodeImage;
export const decodeRgba = binding.decodeRgba;

export default Object.freeze({
  decodeImage,
  decodeRgba,
  encodeText,
  renderJpeg,
  renderPng,
  renderWebp,
});
