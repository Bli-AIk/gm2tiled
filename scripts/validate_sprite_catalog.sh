#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <output_dir>" >&2
    exit 2
fi

output_dir="$1"
manifest="$output_dir/extracted/sprites.json"
catalog_dir="$output_dir/sprite_catalog"

if [ ! -f "$manifest" ]; then
    echo "Missing sprite manifest: $manifest" >&2
    exit 1
fi

if [ ! -d "$catalog_dir" ]; then
    echo "Missing sprite catalog directory: $catalog_dir" >&2
    exit 1
fi

expected="$(mktemp)"
actual="$(mktemp)"
missing="$(mktemp)"
unexpected="$(mktemp)"
trap 'rm -f "$expected" "$actual" "$missing" "$unexpected"' EXIT

jq -r '
    .[]
    | .name as $name
    | .frames
    | to_entries[]
    | "\($name)_\(.key).png"
' "$manifest" | sort -u > "$expected"

find "$catalog_dir" -maxdepth 1 -type f -name '*.png' \
    | sed 's#^.*/##' \
    | sort -u > "$actual"

comm -23 "$expected" "$actual" > "$missing" || true
comm -13 "$expected" "$actual" > "$unexpected" || true

expected_count="$(wc -l < "$expected" | tr -d ' ')"
actual_count="$(wc -l < "$actual" | tr -d ' ')"
missing_count="$(wc -l < "$missing" | tr -d ' ')"
unexpected_count="$(wc -l < "$unexpected" | tr -d ' ')"

echo "Sprite catalog expected: $expected_count"
echo "Sprite catalog actual:   $actual_count"
echo "Missing frames:          $missing_count"
echo "Unexpected files:        $unexpected_count"

if [ "$missing_count" -gt 0 ]; then
    echo
    echo "Missing sprite frames (first 50):"
    sed -n '1,50p' "$missing"
fi

if [ "$unexpected_count" -gt 0 ]; then
    echo
    echo "Unexpected sprite files (first 50):"
    sed -n '1,50p' "$unexpected"
fi

if [ "$missing_count" -ne 0 ] || [ "$unexpected_count" -ne 0 ]; then
    exit 1
fi

echo "Sprite catalog validation OK."
