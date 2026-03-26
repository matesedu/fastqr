export interface BrowserDecodedQr {
  version: number;
  errorCorrection: string;
  mask: number;
  bytes: Uint8Array;
  text: string;
}

export interface BrowserQrCode {
  readonly size: number;
  readonly version: number;
  readonly mask: number;
  errorCorrection(): string;
  modules(): Uint8Array;
  renderRgba(scale: number, border: number): Uint8Array;
  renderSvg(border: number): string;
}

export default function init(
  input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module | Promise<Response>,
): Promise<unknown>;

export declare function encodeText(text: string, errorCorrection?: string): BrowserQrCode;
export declare function decodeRgba(
  rgba: Uint8Array | Uint8ClampedArray | number[],
  width: number,
  height: number,
): BrowserDecodedQr;
export declare function renderToCanvas(
  canvas: HTMLCanvasElement,
  text: string,
  scale: number,
  border: number,
): void;
export declare function decodeCanvas(canvas: HTMLCanvasElement): BrowserDecodedQr;
export declare function decodeVideoFrame(
  video: HTMLVideoElement,
  canvas: HTMLCanvasElement,
): BrowserDecodedQr;
