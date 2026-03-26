import { computed, onMounted, readonly, ref, toValue, watch } from "vue";
import { encodeText } from "fastqr/browser";
import type { MaybeRefOrGetter } from "vue";
import type { BrowserQrCode } from "fastqr/browser";

import {
  DEFAULT_BORDER,
  DEFAULT_SCALE,
  downloadBlob,
  getCanvasContext,
  normalizeInteger,
  toErrorMessage,
} from "./shared";
import type { ElementRef, ErrorCorrection } from "./shared";
import { countLines, summarizeQr } from "./fastqr_canvas";

export type UseFastqrCanvasOptions = {
  border?: MaybeRefOrGetter<number>;
  defaultBorder?: number;
  defaultScale?: number;
  errorCorrection: MaybeRefOrGetter<ErrorCorrection>;
  pngFilename?: string;
  previewCanvas?: ElementRef<HTMLCanvasElement>;
  ready?: MaybeRefOrGetter<boolean>;
  scale?: MaybeRefOrGetter<number>;
  svgFilename?: string;
  text: MaybeRefOrGetter<string>;
};

export function useFastqrCanvas(options: UseFastqrCanvasOptions) {
  const copied = ref(false);
  const previewCanvas = options.previewCanvas ?? ref<HTMLCanvasElement | null>(null);
  const qr = ref<BrowserQrCode | null>(null);
  const renderError = ref("");
  const svgMarkup = ref("");

  const normalizedScale = computed(() =>
    normalizeInteger(
      options.scale == null ? DEFAULT_SCALE : Number(toValue(options.scale)),
      1,
      32,
      options.defaultScale ?? DEFAULT_SCALE,
    ),
  );

  const normalizedBorder = computed(() =>
    normalizeInteger(
      options.border == null ? DEFAULT_BORDER : Number(toValue(options.border)),
      0,
      16,
      options.defaultBorder ?? DEFAULT_BORDER,
    ),
  );

  const renderSummary = computed(() => summarizeQr(qr.value));
  const svgLineCount = computed(() => countLines(svgMarkup.value));

  onMounted(() => {
    renderPreview();
  });

  watch(
    () => [
      options.ready == null ? true : Boolean(toValue(options.ready)),
      toValue(options.text),
      toValue(options.errorCorrection),
      normalizedScale.value,
      normalizedBorder.value,
    ],
    () => {
      renderPreview();
    },
  );

  function renderPreview() {
    const canvas = previewCanvas.value;
    if (!canvas) {
      return;
    }
    if (options.ready != null && !toValue(options.ready)) {
      return;
    }

    try {
      const code = encodeText(toValue(options.text), toValue(options.errorCorrection));
      const scaleValue = normalizedScale.value;
      const borderValue = normalizedBorder.value;
      const rgba = code.renderRgba(scaleValue, borderValue);
      const side = (code.size + borderValue * 2) * scaleValue;
      const view = new Uint8ClampedArray(rgba.buffer, rgba.byteOffset, rgba.byteLength);

      canvas.width = side;
      canvas.height = side;
      const context = getCanvasContext(canvas);
      context.putImageData(new ImageData(view, side, side), 0, 0);

      qr.value = code;
      renderError.value = "";
      svgMarkup.value = code.renderSvg(borderValue);
    } catch (error) {
      qr.value = null;
      renderError.value = toErrorMessage(error);
      svgMarkup.value = "";
    }
  }

  async function copySvg() {
    if (!svgMarkup.value) {
      return;
    }
    if (typeof navigator === "undefined" || !navigator.clipboard) {
      return;
    }

    await navigator.clipboard.writeText(svgMarkup.value);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 1200);
  }

  function downloadPng() {
    const canvas = previewCanvas.value;
    if (!canvas) {
      return;
    }

    canvas.toBlob((blob) => {
      if (!blob) {
        return;
      }
      downloadBlob(blob, options.pngFilename ?? "fastqr.png");
    }, "image/png");
  }

  function downloadSvg() {
    if (!svgMarkup.value) {
      return;
    }

    const blob = new Blob([svgMarkup.value], { type: "image/svg+xml;charset=utf-8" });
    downloadBlob(blob, options.svgFilename ?? "fastqr.svg");
  }

  return {
    copied: readonly(copied),
    copySvg,
    downloadPng,
    downloadSvg,
    previewCanvas,
    renderError: readonly(renderError),
    renderPreview,
    renderSummary,
    svgLineCount,
    svgMarkup: readonly(svgMarkup),
  };
}
