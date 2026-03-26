<script setup lang="ts">
const { bootError, ready } = defineProps<{
  bootError: string;
  ready: boolean;
}>();
</script>

<template>
  <section class="hero">
    <div class="hero-copy">
      <p class="eyebrow">Vue Playground</p>
      <h1>fastqr, tuned for live iteration.</h1>
      <p class="lede">
        Build a code, inspect its structure, export it, then turn right around and verify the decode
        path from file upload or a live camera.
      </p>
    </div>

    <div class="hero-badge">
      <span :class="['status-pill', ready ? 'is-ready' : 'is-waiting']">
        {{ ready ? "WASM ready" : "Booting WASM" }}
      </span>
      <p v-if="bootError">{{ bootError }}</p>
      <p v-else>Powered by the same `fastqr/browser` package shipped to npm.</p>
    </div>
  </section>
</template>
