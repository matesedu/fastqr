<script setup lang="ts">
import { useTemplateRef } from "vue";
import { useFastqrCamera } from "@fastqr/vue";

const { ready } = defineProps<{
  ready: boolean;
}>();
const scratchCanvas = useTemplateRef<HTMLCanvasElement>("scratchCanvas");
const video = useTemplateRef<HTMLVideoElement>("video");

const camera = useFastqrCamera({
  ready: () => ready,
  scratchCanvas,
  video,
});

defineExpose({});
</script>

<template>
  <article class="panel panel-camera">
    <header class="panel-head">
      <div>
        <p class="panel-kicker">Scan</p>
        <h2>Camera loop</h2>
      </div>
      <div class="actions">
        <button
          v-if="!camera.cameraActive"
          class="button button-primary"
          :disabled="!ready"
          type="button"
          @click="camera.startCamera"
        >
          Start camera
        </button>
        <button v-else class="button" type="button" @click="camera.stopCamera">Stop camera</button>
      </div>
    </header>

    <div class="video-frame">
      <video ref="video" autoplay muted playsinline></video>
    </div>
    <p class="decode-meta">{{ camera.cameraLabel }}</p>
    <p v-if="camera.cameraError" class="inline-error">
      {{ camera.cameraError }}
    </p>
    <p v-else-if="camera.cameraText" class="decode-text">
      {{ camera.cameraText }}
    </p>

    <canvas ref="scratchCanvas" class="hidden-canvas"></canvas>
  </article>
</template>
