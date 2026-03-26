import type { Ref, ShallowRef } from "vue";

export type ErrorCorrection = "L" | "M" | "Q" | "H";
export type ElementRef<T> = Readonly<ShallowRef<T | null>> | Ref<T | null>;

export const DEFAULT_ERROR_CORRECTION: ErrorCorrection = "M";
export const DEFAULT_SCALE = 10;
export const DEFAULT_BORDER = 4;

export function isValidEcc(value: string): value is ErrorCorrection {
  return value === "L" || value === "M" || value === "Q" || value === "H";
}

export function getCanvasContext(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) {
    throw new Error("2D canvas context is unavailable");
  }
  return context;
}

export function downloadBlob(blob: Blob, filename: string) {
  if (typeof document === "undefined" || typeof URL === "undefined") {
    return;
  }

  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  setTimeout(() => {
    URL.revokeObjectURL(url);
  }, 0);
}

export type RasterSource = {
  dispose: () => void;
  height: number;
  source: CanvasImageSource;
  width: number;
};

export async function loadRasterSource(file: File): Promise<RasterSource> {
  if (typeof window === "undefined") {
    throw new Error("Raster decoding is only available in a browser context");
  }

  if ("createImageBitmap" in window) {
    const bitmap = await createImageBitmap(file);
    return {
      dispose: () => bitmap.close(),
      height: bitmap.height,
      source: bitmap,
      width: bitmap.width,
    };
  }

  const url = URL.createObjectURL(file);
  try {
    const image = await loadImage(url);
    return {
      dispose: () => URL.revokeObjectURL(url),
      height: image.naturalHeight,
      source: image,
      width: image.naturalWidth,
    };
  } catch (error) {
    URL.revokeObjectURL(url);
    throw error;
  }
}

function loadImage(source: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.decoding = "async";
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error("failed to load image"));
    image.src = source;
  });
}

export function bytesToHex(bytes: Uint8Array): string {
  const hex = "0123456789abcdef";
  let out = "";
  for (let index = 0; index < bytes.length; index += 1) {
    const value = bytes[index];
    out += hex[(value >> 4) & 0x0f];
    out += hex[value & 0x0f];
  }
  return out;
}

export function normalizeInteger(
  value: number,
  min: number,
  max: number,
  fallback: number,
): number {
  const normalized = Number(value);
  if (!Number.isInteger(normalized)) {
    return fallback;
  }
  return Math.min(max, Math.max(min, normalized));
}

export function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}
