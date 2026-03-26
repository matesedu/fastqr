#!/usr/bin/env sh
set -eu

payload="${1:-https://github.com/mates-inc/fastqr}"
output="${2:-./fastqr-shell-example.png}"

vp run cli -- render "$payload" "$output"
vp run cli -- decode "$output"
vp run cli -- encode "$payload"
