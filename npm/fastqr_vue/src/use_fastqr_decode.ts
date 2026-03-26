import { readonly, ref, toValue } from "vue";
import { decodeRgba } from "fastqr/browser";
import type { MaybeRefOrGetter } from "vue";
import type { BrowserDecodedQr } from "fastqr/browser";

import { bytesToHex, getCanvasContext, loadRasterSource, toErrorMessage } from "./shared";
import type { ElementRef, RasterSource } from "./shared";

export type UseFastqrDecodeOptions = {
  decodeCanvas?: ElementRef<HTMLCanvasElement>;
  fileInput?: ElementRef<HTMLInputElement>;
  idleLabel?: string;
  ready?: MaybeRefOrGetter<boolean>;
};

export function useFastqrDecode(options: UseFastqrDecodeOptions = {}) {
  const decodeCanvas = options.decodeCanvas ?? ref<HTMLCanvasElement | null>(null);
  const decodeError = ref("");
  const decodeHex = ref("");
  const decodeLabel = ref(
    options.idleLabel ?? "Drop in a PNG, JPEG, or WebP export to verify a roundtrip.",
  );
  const decodeMeta = ref("");
  const decodeText = ref("");
  const fileInput = options.fileInput ?? ref<HTMLInputElement | null>(null);
  let decodeRequestId = 0;

  function openFilePicker() {
    if (!isReady(options.ready)) {
      decodeError.value = "WASM is still booting.";
      decodeLabel.value = "Wait for the browser package to finish loading.";
      return;
    }

    fileInput.value?.click();
  }

  async function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) {
      return;
    }

    await decodeFromFile(file);
    input.value = "";
  }

  async function decodeFromFile(file: File) {
    const requestId = ++decodeRequestId;
    const canvas = decodeCanvas.value;
    if (!canvas || !isReady(options.ready)) {
      return;
    }

    decodeError.value = "";
    decodeHex.value = "";
    decodeMeta.value = "";
    decodeText.value = "";
    decodeLabel.value = `Reading ${file.name}...`;

    let raster: RasterSource | null = null;
    try {
      raster = await loadRasterSource(file);
      const width = raster.width;
      const height = raster.height;

      canvas.width = width;
      canvas.height = height;

      const context = getCanvasContext(canvas);
      context.clearRect(0, 0, width, height);
      context.drawImage(raster.source, 0, 0, width, height);

      const data = context.getImageData(0, 0, width, height);
      const decoded = decodeRgba(data.data, width, height);
      if (requestId !== decodeRequestId) {
        return;
      }
      assignDecoded(decoded, `Decoded ${file.name}`);
    } catch (error) {
      if (requestId !== decodeRequestId) {
        return;
      }
      decodeError.value = toErrorMessage(error);
      decodeLabel.value = "The selected image did not decode cleanly.";
    } finally {
      raster?.dispose();
    }
  }

  function assignDecoded(decoded: BrowserDecodedQr, label: string) {
    decodeError.value = "";
    decodeHex.value = bytesToHex(decoded.bytes);
    decodeLabel.value = label;
    decodeMeta.value = `v${decoded.version} / ${decoded.errorCorrection} / mask ${decoded.mask}`;
    decodeText.value = decoded.text;
  }

  return {
    decodeCanvas,
    decodeError: readonly(decodeError),
    decodeFromFile,
    decodeHex: readonly(decodeHex),
    decodeLabel: readonly(decodeLabel),
    decodeMeta: readonly(decodeMeta),
    decodeText: readonly(decodeText),
    fileInput,
    handleFileSelect,
    openFilePicker,
  };
}

function isReady(ready: MaybeRefOrGetter<boolean> | undefined) {
  return ready == null ? true : Boolean(toValue(ready));
}
