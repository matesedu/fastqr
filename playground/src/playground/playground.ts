export type Preset = {
  label: string;
  value: string;
};

export const DEFAULT_TEXT = "https://github.com/mates-inc/fastqr";

export const presets: readonly Preset[] = [
  {
    label: "URL",
    value: "https://github.com/mates-inc/fastqr",
  },
  {
    label: "Wi-Fi",
    value: "WIFI:T:WPA;S:fastqr-lab;P:correct-horse-battery-staple;;",
  },
  {
    label: "vCard",
    value:
      "BEGIN:VCARD\nVERSION:3.0\nFN:fastqr Playground\nORG:mates, inc.\nURL:https://github.com/mates-inc/fastqr\nEND:VCARD",
  },
] as const;
export type { ErrorCorrection } from "@fastqr/vue";
export {
  DEFAULT_BORDER,
  DEFAULT_ERROR_CORRECTION,
  DEFAULT_SCALE,
  isValidEcc,
  normalizeInteger,
} from "@fastqr/vue";
