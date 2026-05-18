export type ErrorCorrection = "L" | "M" | "Q" | "H" | "LOW" | "MEDIUM" | "QUARTILE" | "HIGH";

export type HexColor = `#${string}`;

export interface EncodeOptions {
  errorCorrection?: ErrorCorrection;
  minErrorCorrection?: ErrorCorrection;
  minVersion?: number;
  maxVersion?: number;
  boostErrorCorrection?: boolean;
  mask?: number;
}

export interface RenderOptions {
  scale?: number;
  border?: number;
  dark?: HexColor;
  light?: HexColor;
}

export interface ImageOptions extends EncodeOptions, RenderOptions {}

export interface DecodeOptions {
  tryInvert?: boolean;
  maxPixels?: number;
}

export interface QrCode {
  version: number;
  errorCorrection: string;
  mask: number;
  size: number;
  modules: Uint8Array;
}

export interface DecodedQr {
  version: number;
  errorCorrection: string;
  mask: number;
  text?: string;
  bytes: Uint8Array;
}

export declare function encodeText(text: string, options?: ErrorCorrection | EncodeOptions): QrCode;
export declare function renderPng(text: string, options?: ImageOptions): Uint8Array;
export declare function renderJpeg(text: string, options?: ImageOptions): Uint8Array;
export declare function renderWebp(text: string, options?: ImageOptions): Uint8Array;
export declare function decodeImage(bytes: Uint8Array, options?: DecodeOptions): DecodedQr;
export declare function decodeRgba(
  bytes: Uint8Array,
  width: number,
  height: number,
  options?: DecodeOptions,
): DecodedQr;

declare const fastqr: {
  decodeImage: typeof decodeImage;
  decodeRgba: typeof decodeRgba;
  encodeText: typeof encodeText;
  renderJpeg: typeof renderJpeg;
  renderPng: typeof renderPng;
  renderWebp: typeof renderWebp;
};

export default fastqr;
