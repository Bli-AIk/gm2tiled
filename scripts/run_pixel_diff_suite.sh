#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

regress_bin="${GM2TILED_REGRESS_BIN:-./target/release/gm2tiled_regress}"
gm2tiled_bin="${GM2TILED_BIN:-./target/release/gm2tiled}"
summary_csv="${SUMMARY_CSV:-dev/pixel_diff_all_maps.csv}"
skip_export="${SKIP_EXPORT:-0}"

if [ "$#" -eq 0 ]; then
    datasets=(undertale deltarune_ch1 deltarune_ch2 deltarune_ch3 deltarune_ch4)
else
    datasets=("$@")
fi

skip_export_flag=()
if [ "$skip_export" = "1" ]; then
    skip_export_flag+=(--skip-export)
fi

mkdir -p "$(dirname "$summary_csv")"
rm -f "$summary_csv"

first_csv=1
for dataset in "${datasets[@]}"; do
    case "$dataset" in
        undertale)
            input="test_sources/undertale/data.win"
            output="test_output/undertale"
            ;;
        deltarune_ch1)
            input="test_sources/deltarune/ch1.win"
            output="test_output/deltarune_ch1"
            ;;
        deltarune_ch2)
            input="test_sources/deltarune/ch2.win"
            output="test_output/deltarune_ch2"
            ;;
        deltarune_ch3)
            input="test_sources/deltarune/ch3.win"
            output="test_output/deltarune_ch3"
            ;;
        deltarune_ch4)
            input="test_sources/deltarune/ch4.win"
            output="test_output/deltarune_ch4"
            ;;
        *)
            echo "Unknown dataset: $dataset" >&2
            exit 2
            ;;
    esac

    "$regress_bin" \
        --input "$input" \
        --output "$output" \
        --dataset "$dataset" \
        --gm2tiled-bin "$gm2tiled_bin" \
        "${skip_export_flag[@]}"

    dataset_csv="$output/pixel_diff.csv"
    if [ "$first_csv" -eq 1 ]; then
        cp "$dataset_csv" "$summary_csv"
        first_csv=0
    else
        tail -n +2 "$dataset_csv" >> "$summary_csv"
    fi
done

echo "Combined CSV: $summary_csv"
