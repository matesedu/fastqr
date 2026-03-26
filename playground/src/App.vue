<script setup lang="ts">
import { useFastqr } from "@fastqr/vue";

import CameraPanel from "./playground/CameraPanel.vue";
import DecodePanel from "./playground/DecodePanel.vue";
import GeneratorPanel from "./playground/GeneratorPanel.vue";
import PlaygroundHero from "./playground/PlaygroundHero.vue";
import PreviewPanel from "./playground/PreviewPanel.vue";
import { usePlaygroundState } from "./playground/usePlaygroundState";

const fastqr = useFastqr();
const playgroundState = usePlaygroundState();
</script>

<template>
  <main class="playground-shell">
    <PlaygroundHero :boot-error="fastqr.bootError" :ready="fastqr.ready" />

    <section class="grid">
      <GeneratorPanel
        :border="playgroundState.border"
        :error-correction="playgroundState.errorCorrection"
        :scale="playgroundState.scale"
        :text="playgroundState.text"
        @preset:select="playgroundState.setPreset"
        @update:border="playgroundState.setBorder"
        @update:error-correction="playgroundState.setErrorCorrection"
        @update:scale="playgroundState.setScale"
        @update:text="playgroundState.setText"
      />

      <PreviewPanel
        :border="playgroundState.border"
        :error-correction="playgroundState.errorCorrection"
        :ready="fastqr.ready"
        :scale="playgroundState.scale"
        :text="playgroundState.text"
      />

      <DecodePanel :ready="fastqr.ready" />
      <CameraPanel :ready="fastqr.ready" />
    </section>
  </main>
</template>
