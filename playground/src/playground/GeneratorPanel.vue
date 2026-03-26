<script setup lang="ts">
import { computed } from "vue";
import type { ErrorCorrection } from "./playground";

import { presets } from "./playground";

const { border, errorCorrection, scale, text } = defineProps<{
  border: number;
  errorCorrection: ErrorCorrection;
  scale: number;
  text: string;
}>();

const emit = defineEmits<{
  "preset:select": [value: string];
  "update:border": [value: number];
  "update:error-correction": [value: ErrorCorrection];
  "update:scale": [value: number];
  "update:text": [value: string];
}>();

const textModel = computed({
  get: () => text,
  set: (value: string) => emit("update:text", value),
});

const errorCorrectionModel = computed({
  get: () => errorCorrection,
  set: (value: ErrorCorrection) => emit("update:error-correction", value),
});

const scaleModel = computed({
  get: () => scale,
  set: (value: number) => emit("update:scale", value),
});

const borderModel = computed({
  get: () => border,
  set: (value: number) => emit("update:border", value),
});
</script>

<template>
  <article class="panel panel-controls">
    <header class="panel-head">
      <div>
        <p class="panel-kicker">Compose</p>
        <h2>Generator</h2>
      </div>
      <div class="preset-row">
        <button
          v-for="preset in presets"
          :key="preset.label"
          class="chip"
          type="button"
          @click="emit('preset:select', preset.value)"
        >
          {{ preset.label }}
        </button>
      </div>
    </header>

    <label class="field">
      <span>Payload</span>
      <textarea v-model="textModel" rows="8" spellcheck="false"></textarea>
    </label>

    <div class="field-row">
      <label class="field">
        <span>Error correction</span>
        <select v-model="errorCorrectionModel">
          <option value="L">L / low</option>
          <option value="M">M / medium</option>
          <option value="Q">Q / quartile</option>
          <option value="H">H / high</option>
        </select>
      </label>
      <label class="field">
        <span>Scale</span>
        <input v-model.number="scaleModel" max="32" min="1" type="number" />
      </label>
      <label class="field">
        <span>Border</span>
        <input v-model.number="borderModel" max="16" min="0" type="number" />
      </label>
    </div>
  </article>
</template>
