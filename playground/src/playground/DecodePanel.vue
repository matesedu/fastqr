<script setup lang="ts">
import { useTemplateRef } from "vue";
import { useFastqrDecode } from "@fastqr/vue";

const { ready } = defineProps<{
  ready: boolean;
}>();
const decodeCanvas = useTemplateRef<HTMLCanvasElement>("decodeCanvas");
const fileInput = useTemplateRef<HTMLInputElement>("fileInput");

const decode = useFastqrDecode({
  decodeCanvas,
  fileInput,
  ready: () => ready,
});

defineExpose({});
</script>

<template>
  <article class="panel panel-decode">
    <header class="panel-head">
      <div>
        <p class="panel-kicker">Verify</p>
        <h2>Decode from file</h2>
      </div>
    </header>

    <input
      ref="fileInput"
      accept=".png,.jpg,.jpeg,.webp,.wep,image/png,image/jpeg,image/webp"
      class="hidden-input"
      type="file"
      @change="decode.handleFileSelect"
    />

    <button class="drop-zone" :disabled="!ready" type="button" @click="decode.openFilePicker">
      <span>Choose an export</span>
      <small>{{ decode.decodeLabel }}</small>
    </button>

    <div class="decode-output">
      <p class="decode-meta">{{ decode.decodeMeta }}</p>
      <p v-if="decode.decodeError" class="inline-error">
        {{ decode.decodeError }}
      </p>
      <p v-else-if="decode.decodeText" class="decode-text">
        {{ decode.decodeText }}
      </p>
      <pre v-if="decode.decodeHex" class="hex-view">{{ decode.decodeHex }}</pre>
    </div>

    <canvas ref="decodeCanvas" class="hidden-canvas"></canvas>
  </article>
</template>
