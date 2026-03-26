import { onBeforeUnmount, onMounted, readonly, ref } from "vue";
import init from "fastqr/browser";

import { toErrorMessage } from "./shared";

export function useFastqr() {
  const ready = ref(false);
  const bootError = ref("");
  let active = true;

  onBeforeUnmount(() => {
    active = false;
  });

  onMounted(async () => {
    try {
      await init();
      if (!active) {
        return;
      }
      ready.value = true;
    } catch (error) {
      if (!active) {
        return;
      }
      bootError.value = toErrorMessage(error);
    }
  });

  return {
    bootError: readonly(bootError),
    ready: readonly(ready),
  };
}
