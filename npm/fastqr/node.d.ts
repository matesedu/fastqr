export type ErrorCorrection = "L" | "M" | "Q" | "H" | "LOW" | "MEDIUM" | "QUARTILE" | "HIGH";

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

export interface RenderOptions {
  scale?: number;
  border?: number;
}

export declare function encodeText(text: string, errorCorrection?: ErrorCorrection): QrCode;
export declare function renderPng(text: string, render?: RenderOptions): Uint8Array;
export declare function renderJpeg(text: string, render?: RenderOptions): Uint8Array;
export declare function renderWebp(text: string, render?: RenderOptions): Uint8Array;
export declare function decodeImage(bytes: Uint8Array): DecodedQr;
export declare function decodeRgba(bytes: Uint8Array, width: number, height: number): DecodedQr;
