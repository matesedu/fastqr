export type ErrorCorrection = "L" | "M" | "Q" | "H" | "LOW" | "MEDIUM" | "QUARTILE" | "HIGH";

export type HexColor = `#${string}`;

export interface BrowserEncodeOptions {
  errorCorrection?: ErrorCorrection;
  minErrorCorrection?: ErrorCorrection;
  minVersion?: number;
  maxVersion?: number;
  boostErrorCorrection?: boolean;
  mask?: number;
}

export interface BrowserRenderOptions {
  scale?: number;
  border?: number;
  dark?: HexColor;
  light?: HexColor;
}

export interface BrowserImageOptions extends BrowserEncodeOptions, BrowserRenderOptions {}

export interface BrowserDecodeOptions {
  tryInvert?: boolean;
  maxPixels?: number;
}

export interface BrowserDecodedQr {
  version: number;
  errorCorrection: string;
  mask: number;
  bytes: Uint8Array;
  text: string | null;
}

export interface BrowserQrCode {
  readonly size: number;
  readonly version: number;
  readonly mask: number;
  errorCorrection(): string;
  modules(): Uint8Array;
  renderRgba(options?: BrowserRenderOptions): Uint8Array;
  renderRgba(scale: number, border?: number): Uint8Array;
  renderSvg(options?: BrowserRenderOptions): string;
  renderSvg(border?: number): string;
}

export default function init(
  input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module | Promise<Response>,
): Promise<unknown>;

export declare function encodeText(
  text: string,
  options?: ErrorCorrection | BrowserEncodeOptions,
): BrowserQrCode;
export declare function decodeRgba(
  rgba: Uint8Array | Uint8ClampedArray | number[],
  width: number,
  height: number,
  options?: BrowserDecodeOptions,
): BrowserDecodedQr;
export declare function renderToCanvas(
  canvas: HTMLCanvasElement,
  text: string,
  options?: BrowserImageOptions,
): void;
export declare function renderToCanvas(
  canvas: HTMLCanvasElement,
  text: string,
  scale: number,
  border?: number,
): void;
export declare function decodeCanvas(
  canvas: HTMLCanvasElement,
  options?: BrowserDecodeOptions,
): BrowserDecodedQr;
export declare function decodeVideoFrame(
  video: HTMLVideoElement,
  canvas: HTMLCanvasElement,
  options?: BrowserDecodeOptions,
): BrowserDecodedQr;
