use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use clap::Parser;
use serde::Serialize;

#[allow(dead_code)]
#[path = "../convert.rs"]
mod convert;
#[allow(dead_code)]
#[path = "../extract.rs"]
mod extract;
#[allow(dead_code)]
#[path = "../model.rs"]
mod model;
#[path = "../render.rs"]
mod render;
#[allow(dead_code)]
#[path = "../schema.rs"]
mod schema;
#[path = "../textures.rs"]
mod textures;

#[derive(Parser)]
#[command(
    name = "gm2tiled_regress",
    about = "Batch pixel-diff regression for gm2tiled maps"
)]
struct Cli {
    /// Path to data.win file
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for exported maps and regression artifacts
    #[arg(short, long)]
    output: PathBuf,

    /// Rooms to process (comma-separated names, or "all")
    #[arg(short, long, default_value = "all")]
    rooms: String,

    /// Tile size in pixels for map grid
    #[arg(long, default_value_t = 20)]
    tile_size: u32,

    /// Skip re-exporting the Tiled project and reuse an existing output dir
    #[arg(long)]
    skip_export: bool,

    /// Path to the gm2tiled binary used for export
    #[arg(long)]
    gm2tiled_bin: Option<PathBuf>,

    /// Dataset name written to CSV rows
    #[arg(long)]
    dataset: Option<String>,

    /// CSV output path
    #[arg(long)]
    csv: Option<PathBuf>,
}

#[derive(Serialize)]
struct RoomMetricsFile {
    dataset: String,
    room: String,
    status: String,
    ae_pixels: u64,
    ae_percent: f64,
    rmse: f64,
    size_match: bool,
    static_draw_ops: u64,
    gms2_flagged_tiles: u64,
    instance_count: usize,
    enabled_view_count: usize,
    notes: String,
}

struct CsvRow {
    dataset: String,
    room: String,
    status: String,
    ae_pixels: String,
    ae_percent: String,
    rmse: String,
    size_match: String,
    static_draw_ops: String,
    gms2_flagged_tiles: String,
    instance_count: String,
    enabled_view_count: String,
    notes: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let dataset = cli
        .dataset
        .clone()
        .unwrap_or_else(|| dataset_name_from_output(&cli.output));
    let csv_path = cli
        .csv
        .clone()
        .unwrap_or_else(|| cli.output.join("pixel_diff.csv"));

    if !cli.skip_export {
        let gm2tiled_bin = find_gm2tiled_bin(cli.gm2tiled_bin.as_deref())?;
        run_export(&gm2tiled_bin, &cli)?;
    }

    let extract_dir = cli.output.join("extracted");
    let reference_dir = cli.output.join("reference");
    let rendered_dir = cli.output.join("rendered");
    let diff_dir = cli.output.join("diffs");
    let metrics_dir = cli.output.join("metrics");
    for dir in [&reference_dir, &rendered_dir, &diff_dir, &metrics_dir] {
        fs::create_dir_all(dir)?;
    }

    let backgrounds = extract::load_backgrounds(&extract_dir)?;
    let room_names = resolve_room_names(&cli.rooms, &extract_dir)?;
    let mut texture_cache = textures::TexturePageCache::new(&extract_dir.join("textures"));
    let mut region_cache = render::RegionCache::new();

    let mut rows = Vec::new();
    for room_name in room_names {
        match process_room(
            &dataset,
            &room_name,
            &backgrounds,
            &extract_dir,
            &reference_dir,
            &rendered_dir,
            &diff_dir,
            &metrics_dir,
            cli.tile_size,
            &mut texture_cache,
            &mut region_cache,
        ) {
            Ok(row) => rows.push(row),
            Err(error) => rows.push(CsvRow {
                dataset: dataset.clone(),
                room: room_name,
                status: "ERROR".to_string(),
                ae_pixels: String::new(),
                ae_percent: String::new(),
                rmse: String::new(),
                size_match: String::new(),
                static_draw_ops: String::new(),
                gms2_flagged_tiles: String::new(),
                instance_count: String::new(),
                enabled_view_count: String::new(),
                notes: error.to_string(),
            }),
        }
    }

    write_csv(&csv_path, &rows)?;
    println!("Wrote pixel-diff CSV: {}", csv_path.display());
    println!("Processed {} rooms for dataset '{}'.", rows.len(), dataset);
    Ok(())
}

fn process_room(
    dataset: &str,
    room_name: &str,
    backgrounds: &HashMap<String, schema::BackgroundDef>,
    extract_dir: &Path,
    reference_dir: &Path,
    rendered_dir: &Path,
    diff_dir: &Path,
    metrics_dir: &Path,
    tile_size: u32,
    texture_cache: &mut textures::TexturePageCache,
    region_cache: &mut render::RegionCache,
) -> anyhow::Result<CsvRow> {
    let room = extract::load_room(extract_dir, room_name)?;
    let (reference_image, reference_stats) =
        render::render_reference_room_static(&room, backgrounds, texture_cache, region_cache)?;
    let (tiled_map, tilesets, _) = convert::convert_room(&room, backgrounds, tile_size)?;
    let rendered_image = render::render_tiled_map_static(
        &tiled_map,
        &tilesets,
        backgrounds,
        room.width,
        room.height,
        texture_cache,
        region_cache,
    )?;
    let diff = render::compare_images(&reference_image, &rendered_image);
    let notes = build_notes(&reference_stats);
    let status = determine_status(&reference_stats, &diff.metrics);

    let reference_path = reference_dir.join(format!("{room_name}.png"));
    let rendered_path = rendered_dir.join(format!("{room_name}.png"));
    let diff_path = diff_dir.join(format!("{room_name}.png"));
    reference_image
        .save(&reference_path)
        .with_context(|| format!("Failed to save {reference_path:?}"))?;
    rendered_image
        .save(&rendered_path)
        .with_context(|| format!("Failed to save {rendered_path:?}"))?;
    diff.diff_image
        .save(&diff_path)
        .with_context(|| format!("Failed to save {diff_path:?}"))?;

    let metrics_file = RoomMetricsFile {
        dataset: dataset.to_string(),
        room: room_name.to_string(),
        status: status.to_string(),
        ae_pixels: diff.metrics.ae_pixels,
        ae_percent: diff.metrics.ae_percent,
        rmse: diff.metrics.rmse,
        size_match: diff.metrics.size_match,
        static_draw_ops: reference_stats.static_draw_ops,
        gms2_flagged_tiles: reference_stats.gms2_flagged_tiles,
        instance_count: reference_stats.instance_count,
        enabled_view_count: reference_stats.enabled_view_count,
        notes: notes.clone(),
    };
    let metrics_path = metrics_dir.join(format!("{room_name}.json"));
    fs::write(&metrics_path, serde_json::to_vec_pretty(&metrics_file)?)
        .with_context(|| format!("Failed to write {metrics_path:?}"))?;

    Ok(CsvRow {
        dataset: dataset.to_string(),
        room: room_name.to_string(),
        status: status.to_string(),
        ae_pixels: diff.metrics.ae_pixels.to_string(),
        ae_percent: format!("{:.8}", diff.metrics.ae_percent),
        rmse: format!("{:.8}", diff.metrics.rmse),
        size_match: diff.metrics.size_match.to_string(),
        static_draw_ops: reference_stats.static_draw_ops.to_string(),
        gms2_flagged_tiles: reference_stats.gms2_flagged_tiles.to_string(),
        instance_count: reference_stats.instance_count.to_string(),
        enabled_view_count: reference_stats.enabled_view_count.to_string(),
        notes,
    })
}

fn run_export(gm2tiled_bin: &Path, cli: &Cli) -> anyhow::Result<()> {
    let output = Command::new(gm2tiled_bin)
        .arg("--input")
        .arg(&cli.input)
        .arg("--output")
        .arg(&cli.output)
        .arg("--rooms")
        .arg(&cli.rooms)
        .arg("--tile-size")
        .arg(cli.tile_size.to_string())
        .output()
        .with_context(|| format!("Failed to run export binary {:?}", gm2tiled_bin))?;

    if !output.status.success() {
        eprintln!(
            "gm2tiled stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "gm2tiled stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        anyhow::bail!("gm2tiled export failed with status {}", output.status);
    }
    Ok(())
}

fn find_gm2tiled_bin(explicit: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }
    if let Ok(path) = std::env::var("GM2TILED_BIN") {
        return Ok(PathBuf::from(path));
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let sibling = dir.join("gm2tiled");
        if sibling.exists() {
            return Ok(sibling);
        }
    }
    let cwd_candidate = std::env::current_dir()?.join("target/debug/gm2tiled");
    if cwd_candidate.exists() {
        return Ok(cwd_candidate);
    }
    anyhow::bail!("Could not find gm2tiled binary. Use --gm2tiled-bin or GM2TILED_BIN.");
}

fn resolve_room_names(rooms_arg: &str, extract_dir: &Path) -> anyhow::Result<Vec<String>> {
    if rooms_arg == "all" {
        return extract::list_rooms(extract_dir);
    }
    Ok(rooms_arg.split(',').map(|s| s.trim().to_string()).collect())
}

fn determine_status(
    reference_stats: &render::ReferenceStats,
    metrics: &render::PixelDiffMetrics,
) -> &'static str {
    if reference_stats.static_draw_ops == 0 {
        return "NO_STATIC_MAP";
    }
    if reference_stats.gms2_flagged_tiles > 0 {
        return "UNSUPPORTED_FLAGS";
    }
    if !metrics.size_match {
        return "FAIL_SIZE";
    }
    if metrics.ae_pixels == 0 {
        "PASS"
    } else {
        "FAIL"
    }
}

fn build_notes(reference_stats: &render::ReferenceStats) -> String {
    let mut notes = Vec::new();
    if reference_stats.instance_count > 0 {
        notes.push(format!(
            "instances_skipped={}",
            reference_stats.instance_count
        ));
    }
    if reference_stats.enabled_view_count > 0 {
        notes.push(format!(
            "views_ignored={}",
            reference_stats.enabled_view_count
        ));
    }
    if reference_stats.gms2_flagged_tiles > 0 {
        notes.push(format!(
            "gms2_flags_ignored={}",
            reference_stats.gms2_flagged_tiles
        ));
    }
    if reference_stats.static_draw_ops == 0 {
        notes.push("no_static_tiles".to_string());
    }
    notes.join(";")
}

fn write_csv(path: &Path, rows: &[CsvRow]) -> anyhow::Result<()> {
    let mut content = String::from(
        "dataset,room,status,ae_pixels,ae_percent,rmse,size_match,static_draw_ops,gms2_flagged_tiles,instance_count,enabled_view_count,notes\n",
    );
    for row in rows {
        let fields = [
            &row.dataset,
            &row.room,
            &row.status,
            &row.ae_pixels,
            &row.ae_percent,
            &row.rmse,
            &row.size_match,
            &row.static_draw_ops,
            &row.gms2_flagged_tiles,
            &row.instance_count,
            &row.enabled_view_count,
            &row.notes,
        ];
        let escaped = fields
            .iter()
            .map(|field| csv_escape(field))
            .collect::<Vec<_>>()
            .join(",");
        content.push_str(&escaped);
        content.push('\n');
    }
    fs::write(path, content).with_context(|| format!("Failed to write {path:?}"))
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn dataset_name_from_output(output: &Path) -> String {
    output
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("dataset")
        .to_string()
}
