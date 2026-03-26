<script setup lang="ts">
import { useTemplateRef } from "vue";
import { useFastqrCanvas } from "@fastqr/vue";
import type { ErrorCorrection } from "./playground";

const { border, errorCorrection, ready, scale, text } = defineProps<{
  border: number;
  errorCorrection: ErrorCorrection;
  ready: boolean;
  scale: number;
  text: string;
}>();
const previewCanvas = useTemplateRef<HTMLCanvasElement>("previewCanvas");

const preview = useFastqrCanvas({
  border: () => border,
  errorCorrection: () => errorCorrection,
  pngFilename: "fastqr-playground.png",
  previewCanvas,
  ready: () => ready,
  scale: () => scale,
  svgFilename: "fastqr-playground.svg",
  text: () => text,
});

defineExpose({});
</script>

<template>
  <article class="panel panel-preview">
    <header class="panel-head">
      <div>
        <p class="panel-kicker">Inspect</p>
        <h2>Live preview</h2>
      </div>
      <div class="stat-grid">
        <div class="stat-card">
          <span>Version</span>
          <strong>{{ preview.renderSummary.version }}</strong>
        </div>
        <div class="stat-card">
          <span>Matrix</span>
          <strong>{{ preview.renderSummary.size }}</strong>
        </div>
        <div class="stat-card">
          <span>Mask</span>
          <strong>{{ preview.renderSummary.mask }}</strong>
        </div>
        <div class="stat-card">
          <span>ECC</span>
          <strong>{{ preview.renderSummary.errorCorrection }}</strong>
        </div>
        <div class="stat-card">
          <span>Dark modules</span>
          <strong>{{ preview.renderSummary.darkModules }}</strong>
        </div>
        <div class="stat-card">
          <span>SVG lines</span>
          <strong>{{ preview.svgLineCount }}</strong>
        </div>
      </div>
    </header>

    <div class="preview-stack">
      <div class="canvas-stage">
        <canvas ref="previewCanvas"></canvas>
      </div>

      <div class="svg-stage">
        <div class="svg-preview" v-html="preview.svgMarkup"></div>
        <pre class="svg-source">{{ preview.svgMarkup }}</pre>
      </div>
    </div>

    <div class="actions">
      <button class="button button-primary" type="button" @click="preview.downloadPng">
        Download PNG
      </button>
      <button class="button" type="button" @click="preview.downloadSvg">Download SVG</button>
      <button class="button" type="button" @click="preview.copySvg">
        {{ preview.copied ? "SVG copied" : "Copy SVG" }}
      </button>
    </div>

    <p v-if="preview.renderError" class="inline-error">
      {{ preview.renderError }}
    </p>
  </article>
</template>
