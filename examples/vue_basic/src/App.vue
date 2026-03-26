<script setup lang="ts">
import { ref, useTemplateRef } from "vue";
import { useFastqr, useFastqrCanvas } from "@fastqr/vue";
import type { ErrorCorrection } from "@fastqr/vue";

const text = ref("https://github.com/mates-inc/fastqr");
const errorCorrection = ref<ErrorCorrection>("M");
const scale = ref(10);
const previewCanvas = useTemplateRef<HTMLCanvasElement>("previewCanvas");

const fastqr = useFastqr();
const preview = useFastqrCanvas({
  errorCorrection,
  pngFilename: "fastqr-vue-example.png",
  previewCanvas,
  ready: fastqr.ready,
  scale,
  text,
});

defineExpose({});
</script>

<template>
  <main class="example-shell">
    <section class="example-card">
      <p class="eyebrow">Vue Example</p>
      <h1>@fastqr/vue in one file.</h1>
      <p class="lede">This sample uses `useFastqr()` and `useFastqrCanvas()` directly.</p>
      <p v-if="fastqr.bootError" class="inline-error">
        {{ fastqr.bootError }}
      </p>

      <label class="field">
        <span>Payload</span>
        <textarea v-model="text" rows="7" spellcheck="false"></textarea>
      </label>

      <div class="field-row">
        <label class="field">
          <span>ECC</span>
          <select v-model="errorCorrection">
            <option value="L">L</option>
            <option value="M">M</option>
            <option value="Q">Q</option>
            <option value="H">H</option>
          </select>
        </label>
        <label class="field">
          <span>Scale</span>
          <input v-model.number="scale" max="24" min="1" type="number" />
        </label>
      </div>

      <div class="actions">
        <button
          class="button button-primary"
          :disabled="!fastqr.ready"
          type="button"
          @click="preview.downloadPng"
        >
          Download PNG
        </button>
        <button class="button" :disabled="!fastqr.ready" type="button" @click="preview.copySvg">
          {{ preview.copied ? "SVG copied" : "Copy SVG" }}
        </button>
      </div>
    </section>

    <section class="example-card">
      <div class="stat-row">
        <span>{{ preview.renderSummary.version }}</span>
        <span>{{ preview.renderSummary.errorCorrection }}</span>
        <span>{{ preview.renderSummary.mask }}</span>
      </div>
      <div class="canvas-frame">
        <canvas ref="previewCanvas"></canvas>
      </div>
      <p v-if="preview.renderError" class="inline-error">
        {{ preview.renderError }}
      </p>
      <pre>{{ preview.svgMarkup }}</pre>
    </section>
  </main>
</template>
