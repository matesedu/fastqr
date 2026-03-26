import type { BrowserQrCode } from "fastqr/browser";

export type RenderSummary = {
  darkModules: string;
  errorCorrection: string;
  mask: string;
  size: string;
  version: string;
};

export const emptyRenderSummary: RenderSummary = {
  darkModules: "–",
  errorCorrection: "–",
  mask: "–",
  size: "–",
  version: "–",
};

export const countLines = (value: string): string => {
  if (!value) {
    return "–";
  }

  let count = 1;
  for (let index = 0; index < value.length; index += 1) {
    if (value.charCodeAt(index) === 10) {
      count += 1;
    }
  }

  return String(count);
};

export const summarizeQr = (code: BrowserQrCode | null): RenderSummary => {
  if (!code) {
    return emptyRenderSummary;
  }

  const modules = code.modules();
  let darkModules = 0;
  for (let index = 0; index < modules.length; index += 1) {
    darkModules += modules[index];
  }

  return {
    darkModules: String(darkModules),
    errorCorrection: code.errorCorrection(),
    mask: String(code.mask),
    size: `${code.size} x ${code.size}`,
    version: String(code.version),
  };
};
