#!/usr/bin/env sh
set -eu

output_dir="${1:-./fastqr-batch}"
mkdir -p "$output_dir"

index=0
for payload in \
  "https://github.com/mates-inc/fastqr" \
  "WIFI:T:WPA;S:fastqr-lab;P:correct-horse-battery-staple;;" \
  "BEGIN:VCARD
VERSION:3.0
FN:fastqr Batch
ORG:mates, inc.
END:VCARD"
do
  file="$output_dir/code-$index.png"
  vp run cli -- render "$payload" "$file"
  index=$((index + 1))
done
