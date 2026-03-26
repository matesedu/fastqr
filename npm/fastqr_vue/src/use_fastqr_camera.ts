import { onBeforeUnmount, readonly, ref, toValue } from "vue";
import { decodeVideoFrame } from "fastqr/browser";
import type { MaybeRefOrGetter } from "vue";

import { toErrorMessage } from "./shared";
import type { ElementRef } from "./shared";

export type UseFastqrCameraOptions = {
  ready?: MaybeRefOrGetter<boolean>;
  scratchCanvas?: ElementRef<HTMLCanvasElement>;
  video?: ElementRef<HTMLVideoElement>;
};

export function useFastqrCamera(options: UseFastqrCameraOptions = {}) {
  const cameraActive = ref(false);
  const cameraError = ref("");
  const cameraLabel = ref("Point a camera at a QR code to scan in place.");
  const cameraText = ref("");
  const scratchCanvas = options.scratchCanvas ?? ref<HTMLCanvasElement | null>(null);
  const video = options.video ?? ref<HTMLVideoElement | null>(null);

  let cameraFrame = 0;
  let cameraRequestId = 0;
  let cameraStream: MediaStream | null = null;

  onBeforeUnmount(() => {
    stopCamera();
  });

  async function startCamera() {
    const requestId = ++cameraRequestId;
    if (!isReady(options.ready) || cameraActive.value) {
      if (!isReady(options.ready)) {
        cameraError.value = "WASM is still booting.";
        cameraLabel.value = "Wait for the browser package to finish loading.";
      }
      return;
    }
    if (typeof navigator === "undefined" || !navigator.mediaDevices) {
      cameraError.value = "Media devices are unavailable in this environment.";
      cameraLabel.value = "Camera access is only available in a browser context.";
      return;
    }

    cameraError.value = "";
    cameraText.value = "";
    cameraLabel.value = "Requesting a rear-facing camera...";

    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: false,
        video: {
          facingMode: { ideal: "environment" },
        },
      });
      if (requestId !== cameraRequestId) {
        stopTracks(stream);
        return;
      }

      if (!video.value) {
        throw new Error("video element is unavailable");
      }

      cameraStream = stream;
      video.value.srcObject = cameraStream;
      await video.value.play();
      if (requestId !== cameraRequestId) {
        stopCamera();
        return;
      }
      cameraActive.value = true;
      cameraLabel.value = "Scanning live frames. Hold steady for a lock.";
      scheduleCameraFrame();
    } catch (error) {
      stopCamera();
      cameraError.value = toErrorMessage(error);
      cameraLabel.value = "Camera access failed.";
    }
  }

  function stopCamera() {
    cameraRequestId += 1;
    cameraActive.value = false;

    if (cameraFrame !== 0) {
      cancelAnimationFrame(cameraFrame);
      cameraFrame = 0;
    }

    if (video.value) {
      video.value.pause();
      video.value.srcObject = null;
    }

    if (!cameraStream) {
      return;
    }

    stopTracks(cameraStream);
    cameraStream = null;
  }

  function scheduleCameraFrame() {
    if (typeof requestAnimationFrame === "undefined") {
      return;
    }
    cameraFrame = requestAnimationFrame(scanCameraFrame);
  }

  function scanCameraFrame() {
    if (!cameraActive.value || !video.value || !scratchCanvas.value) {
      return;
    }

    try {
      const decoded = decodeVideoFrame(video.value, scratchCanvas.value);
      if (decoded.text && decoded.text !== cameraText.value) {
        cameraText.value = decoded.text;
        cameraLabel.value = `Camera lock: v${decoded.version} / ${decoded.errorCorrection} / mask ${decoded.mask}`;
      }
    } catch {
      // Keep scanning until we have a clean frame.
    }

    scheduleCameraFrame();
  }

  return {
    cameraActive: readonly(cameraActive),
    cameraError: readonly(cameraError),
    cameraLabel: readonly(cameraLabel),
    cameraText: readonly(cameraText),
    scratchCanvas,
    startCamera,
    stopCamera,
    video,
  };
}

function isReady(ready: MaybeRefOrGetter<boolean> | undefined) {
  return ready == null ? true : Boolean(toValue(ready));
}

function stopTracks(stream: MediaStream) {
  const tracks = stream.getTracks();
  for (let index = 0; index < tracks.length; index += 1) {
    tracks[index].stop();
  }
}
