import init, {
  decodeRgba as decodeRgbaWasm,
  encodeText as encodeTextWasm,
} from "./wasm/fastqr_wasm.js";

const DEFAULT_SCALE = 8;
const DEFAULT_BORDER = 4;
const DEFAULT_DARK = [0, 0, 0, 255];
const DEFAULT_LIGHT = [255, 255, 255, 255];

export default init;

export function encodeText(text, options) {
  return wrapQrCode(encodeTextWasm(text, normalizeEncodeOptions(options)));
}

export function decodeRgba(rgba, width, height, options) {
  return decodeRgbaWasm(rgba, width, height, normalizeDecodeOptions(options));
}

export function renderToCanvas(canvas, text, optionsOrScale, border) {
  const code = encodeText(text, normalizeEncodeOptions(optionsOrScale));
  const render = normalizeRenderOptions(optionsOrScale, border);
  const rgba = code.renderRgba(render);
  const side = (code.size + render.border * 2) * render.scale;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) {
    throw new Error("2D canvas context is unavailable");
  }
  canvas.width = side;
  canvas.height = side;
  context.putImageData(
    new ImageData(new Uint8ClampedArray(rgba.buffer, rgba.byteOffset, rgba.byteLength), side, side),
    0,
    0,
  );
}

export function decodeCanvas(canvas, options) {
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) {
    throw new Error("2D canvas context is unavailable");
  }
  const data = context.getImageData(0, 0, canvas.width, canvas.height);
  return decodeRgba(data.data, canvas.width, canvas.height, options);
}

export function decodeVideoFrame(video, canvas, options) {
  const width = video.videoWidth;
  const height = video.videoHeight;
  if (width === 0 || height === 0) {
    throw new Error("video element has no frame yet");
  }
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) {
    throw new Error("2D canvas context is unavailable");
  }
  context.drawImage(video, 0, 0);
  return decodeCanvas(canvas, options);
}

function wrapQrCode(code) {
  return {
    get mask() {
      return code.mask;
    },
    get size() {
      return code.size;
    },
    get version() {
      return code.version;
    },
    errorCorrection() {
      return code.errorCorrection();
    },
    modules() {
      return code.modules();
    },
    renderRgba(optionsOrScale, border) {
      const render = normalizeRenderOptions(optionsOrScale, border);
      const rgba = code.renderRgba(render.scale, render.border);
      return recolorRgba(rgba, render.dark, render.light);
    },
    renderSvg(optionsOrBorder) {
      const render = normalizeRenderOptions(optionsOrBorder);
      const svg = code.renderSvg(render.border);
      return svg
        .replace('fill="#fff"', `fill="${rgbaToHex(render.light)}"`)
        .replace('fill="#000"', `fill="${rgbaToHex(render.dark)}"`);
    },
  };
}

function normalizeEncodeOptions(options) {
  if (options == null || typeof options === "string") {
    return options;
  }
  return {
    boostErrorCorrection: options.boostErrorCorrection,
    errorCorrection: options.errorCorrection,
    mask: options.mask,
    maxVersion: options.maxVersion,
    minErrorCorrection: options.minErrorCorrection,
    minVersion: options.minVersion,
  };
}

function normalizeDecodeOptions(options) {
  if (options == null) {
    return undefined;
  }
  return {
    maxPixels: options.maxPixels,
    tryInvert: options.tryInvert,
  };
}

function normalizeRenderOptions(optionsOrScale, border) {
  if (typeof optionsOrScale === "number") {
    return {
      border: border ?? DEFAULT_BORDER,
      dark: DEFAULT_DARK,
      light: DEFAULT_LIGHT,
      scale: optionsOrScale,
    };
  }
  const options = optionsOrScale ?? {};
  return {
    border: options.border ?? DEFAULT_BORDER,
    dark: parseHexColor(options.dark, DEFAULT_DARK, "dark"),
    light: parseHexColor(options.light, DEFAULT_LIGHT, "light"),
    scale: options.scale ?? DEFAULT_SCALE,
  };
}

function recolorRgba(rgba, dark, light) {
  if (sameColor(dark, DEFAULT_DARK) && sameColor(light, DEFAULT_LIGHT)) {
    return rgba;
  }
  const recolored = new Uint8Array(rgba);
  for (let index = 0; index < recolored.length; index += 4) {
    const source = recolored[index] < 128 ? dark : light;
    recolored[index] = source[0];
    recolored[index + 1] = source[1];
    recolored[index + 2] = source[2];
    recolored[index + 3] = source[3];
  }
  return recolored;
}

function parseHexColor(value, fallback, name) {
  if (value == null) {
    return fallback;
  }
  if (typeof value !== "string" || !value.startsWith("#")) {
    throw new Error(`${name} must be a hex color string`);
  }
  const hex = value.slice(1);
  if (hex.length === 3) {
    return [
      parseHexNibble(hex[0], name) * 17,
      parseHexNibble(hex[1], name) * 17,
      parseHexNibble(hex[2], name) * 17,
      255,
    ];
  }
  if (hex.length === 6 || hex.length === 8) {
    return [
      parseHexByte(hex.slice(0, 2), name),
      parseHexByte(hex.slice(2, 4), name),
      parseHexByte(hex.slice(4, 6), name),
      hex.length === 8 ? parseHexByte(hex.slice(6, 8), name) : 255,
    ];
  }
  throw new Error(`${name} must be #rgb, #rrggbb, or #rrggbbaa`);
}

function parseHexByte(value, name) {
  return parseHexNibble(value[0], name) * 16 + parseHexNibble(value[1], name);
}

function parseHexNibble(value, name) {
  const nibble = Number.parseInt(value, 16);
  if (!Number.isInteger(nibble)) {
    throw new Error(`${name} must be a hex color string`);
  }
  return nibble;
}

function rgbaToHex(color) {
  const hex = color.map((channel) => channel.toString(16).padStart(2, "0")).join("");
  return color[3] === 255 ? `#${hex.slice(0, 6)}` : `#${hex}`;
}

function sameColor(left, right) {
  return (
    left[0] === right[0] && left[1] === right[1] && left[2] === right[2] && left[3] === right[3]
  );
}
