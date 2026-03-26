import type { ErrorCorrection } from "./playground";

import {
  DEFAULT_BORDER,
  DEFAULT_ERROR_CORRECTION,
  DEFAULT_SCALE,
  DEFAULT_TEXT,
  isValidEcc,
  normalizeInteger,
} from "./playground";

export type PlaygroundState = {
  border: number;
  errorCorrection: ErrorCorrection;
  scale: number;
  text: string;
};

export const readPlaygroundState = (url: URL): PlaygroundState => {
  const textValue = url.searchParams.get("text");
  const eccValue = url.searchParams.get("ecc");
  const scaleValue = url.searchParams.get("scale");
  const borderValue = url.searchParams.get("border");

  return {
    border: readIntegerParam(borderValue, 0, 16, DEFAULT_BORDER),
    errorCorrection: readErrorCorrection(eccValue),
    scale: readIntegerParam(scaleValue, 1, 32, DEFAULT_SCALE),
    text: textValue ?? DEFAULT_TEXT,
  };
};

export const writePlaygroundState = (url: URL, state: PlaygroundState): string => {
  const nextUrl = new URL(url.href);

  nextUrl.searchParams.set("text", state.text);
  nextUrl.searchParams.set("ecc", state.errorCorrection);
  nextUrl.searchParams.set("scale", String(state.scale));
  nextUrl.searchParams.set("border", String(state.border));

  return nextUrl.toString();
};

const readErrorCorrection = (value: string | null): ErrorCorrection => {
  if (value && isValidEcc(value)) {
    return value;
  }

  return DEFAULT_ERROR_CORRECTION;
};

const readIntegerParam = (
  value: string | null,
  min: number,
  max: number,
  fallback: number,
): number => {
  if (value == null || value === "") {
    return fallback;
  }

  const parsedValue = Number(value);
  if (!Number.isInteger(parsedValue)) {
    return fallback;
  }

  return normalizeInteger(parsedValue, min, max, fallback);
};
