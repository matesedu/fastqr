import { readonly, ref, watch } from "vue";
import type { ErrorCorrection } from "./playground";

import {
  DEFAULT_BORDER,
  DEFAULT_ERROR_CORRECTION,
  DEFAULT_SCALE,
  DEFAULT_TEXT,
  normalizeInteger,
} from "./playground";
import { readPlaygroundState, writePlaygroundState } from "./playground_state";

export function usePlaygroundState() {
  const initialState = resolveInitialState();
  const text = ref(initialState.text);
  const errorCorrection = ref<ErrorCorrection>(initialState.errorCorrection);
  const scale = ref(initialState.scale);
  const border = ref(initialState.border);

  watch([text, errorCorrection, scale, border], () => {
    syncUrlState({
      border: border.value,
      errorCorrection: errorCorrection.value,
      scale: scale.value,
      text: text.value,
    });
  });

  function setBorder(value: number) {
    border.value = normalizeInteger(value, 0, 16, DEFAULT_BORDER);
  }

  function setErrorCorrection(value: ErrorCorrection) {
    errorCorrection.value = value;
  }

  function setPreset(value: string) {
    text.value = value;
  }

  function setScale(value: number) {
    scale.value = normalizeInteger(value, 1, 32, DEFAULT_SCALE);
  }

  function setText(value: string) {
    text.value = value;
  }

  return {
    border: readonly(border),
    errorCorrection: readonly(errorCorrection),
    scale: readonly(scale),
    setBorder,
    setErrorCorrection,
    setPreset,
    setScale,
    setText,
    text: readonly(text),
  };
}

function resolveInitialState() {
  if (typeof window === "undefined") {
    return {
      border: DEFAULT_BORDER,
      errorCorrection: DEFAULT_ERROR_CORRECTION,
      scale: DEFAULT_SCALE,
      text: DEFAULT_TEXT,
    };
  }

  return readPlaygroundState(new URL(window.location.href));
}

function syncUrlState(state: {
  border: number;
  errorCorrection: ErrorCorrection;
  scale: number;
  text: string;
}) {
  if (typeof window === "undefined") {
    return;
  }

  const url = new URL(window.location.href);
  const nextHref = writePlaygroundState(url, state);
  window.history.replaceState({}, "", nextHref);
}
