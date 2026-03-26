import init, { decodeVideoFrame, renderToCanvas } from "fastqr/browser";

const textInput = document.querySelector<HTMLInputElement>("#text-input");
const renderButton = document.querySelector<HTMLButtonElement>("#render-button");
const qrCanvas = document.querySelector<HTMLCanvasElement>("#qr-canvas");
const cameraButton = document.querySelector<HTMLButtonElement>("#camera-button");
const video = document.querySelector<HTMLVideoElement>("#video");
const scratch = document.querySelector<HTMLCanvasElement>("#scratch");
const scanResult = document.querySelector<HTMLDivElement>("#scan-result");

if (
  !textInput ||
  !renderButton ||
  !qrCanvas ||
  !cameraButton ||
  !video ||
  !scratch ||
  !scanResult
) {
  throw new Error("demo DOM is incomplete");
}

await init();

const render = () => {
  renderToCanvas(qrCanvas, textInput.value, 8, 4);
};

renderButton.addEventListener("click", render);
textInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    render();
  }
});
render();

cameraButton.addEventListener("click", async () => {
  const stream = await navigator.mediaDevices.getUserMedia({
    video: { facingMode: "environment" },
    audio: false,
  });
  video.srcObject = stream;

  const tick = () => {
    try {
      const decoded = decodeVideoFrame(video, scratch) as { text: string };
      if (decoded.text) {
        scanResult.textContent = decoded.text;
      }
    } catch {
      // Ignore frames that do not decode cleanly.
    }
    requestAnimationFrame(tick);
  };

  requestAnimationFrame(tick);
});
