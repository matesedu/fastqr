import init, { encodeText } from "fastqr/browser";

import "./style.css";

const canvas = document.querySelector<HTMLCanvasElement>("#canvas");
const payload = document.querySelector<HTMLTextAreaElement>("#payload");
const ecc = document.querySelector<HTMLSelectElement>("#ecc");
const scale = document.querySelector<HTMLInputElement>("#scale");
const meta = document.querySelector<HTMLParagraphElement>("#meta");
const svg = document.querySelector<HTMLElement>("#svg");

if (!canvas || !payload || !ecc || !scale || !meta || !svg) {
  throw new Error("example DOM is incomplete");
}

await init();

const context = canvas.getContext("2d");
if (!context) {
  throw new Error("2D canvas context is unavailable");
}

payload.addEventListener("input", render);
ecc.addEventListener("change", render);
scale.addEventListener("input", render);

render();

function render() {
  const scaleValue = clampInteger(Number.parseInt(scale.value, 10), 1, 24, 10);
  const code = encodeText(payload.value, ecc.value);
  const rgba = code.renderRgba(scaleValue, 4);
  const side = (code.size + 8) * scaleValue;
  const view = new Uint8ClampedArray(rgba.buffer, rgba.byteOffset, rgba.byteLength);

  canvas.width = side;
  canvas.height = side;
  context.putImageData(new ImageData(view, side, side), 0, 0);

  meta.textContent = `v${code.version} / ${code.errorCorrection()} / mask ${code.mask}`;
  svg.textContent = code.renderSvg(4);
}

function clampInteger(value: number, min: number, max: number, fallback: number) {
  if (!Number.isInteger(value)) {
    return fallback;
  }
  return Math.min(max, Math.max(min, value));
}
