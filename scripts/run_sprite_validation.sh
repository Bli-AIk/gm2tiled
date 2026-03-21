#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
    echo "Usage: $0 <input_win> [room]" >&2
    exit 2
fi

script_dir="$(cd "$(dirname "$0")" && pwd)"
input_win="$1"
room="${2:-all}"
gm2tiled_bin="${GM2TILED_BIN:-./target/debug/gm2tiled}"
output_dir="$(mktemp -d /tmp/gm2tiled_validate.XXXXXX)"

echo "Validation output dir: $output_dir"

"$gm2tiled_bin" \
    --input "$input_win" \
    --output "$output_dir" \
    --rooms "$room" \
    --export-all-sprites

"$script_dir/validate_sprite_catalog.sh" "$output_dir"
