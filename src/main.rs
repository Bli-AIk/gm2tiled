use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;

mod convert;
mod export;
mod extract;
mod model;
mod schema;
mod textures;

#[derive(Parser)]
#[command(name = "gm2tiled", about = "Convert GameMaker data.win to Tiled .tmx")]
struct Cli {
    /// Path to data.win file
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for .tmx files and assets
    #[arg(short, long)]
    output: PathBuf,

    /// Rooms to convert (comma-separated names, or "all")
    #[arg(short, long, default_value = "all")]
    rooms: String,

    /// Tile size in pixels for map grid (default: 20 for Undertale)
    #[arg(long, default_value_t = 20)]
    tile_size: u32,

    /// Skip utmt extraction (use existing extracted data)
    #[arg(long)]
    skip_extract: bool,

    /// Export the full sprite catalog into sprite_catalog/
    #[arg(long)]
    export_all_sprites: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let extract_dir = cli.output.join("extracted");
    let tilesets_dir = cli.output.join("tilesets");
    let textures_dir = cli.output.join("textures");
    let sprites_dir = cli.output.join("sprites");
    let tile_objects_dir = cli.output.join("tile_objects");
    let sprite_catalog_dir = cli.output.join("sprite_catalog");

    for dir in [
        &cli.output,
        &extract_dir,
        &tilesets_dir,
        &textures_dir,
        &sprites_dir,
        &tile_objects_dir,
    ] {
        std::fs::create_dir_all(dir)?;
    }
    if cli.export_all_sprites {
        std::fs::create_dir_all(&sprite_catalog_dir)?;
    }

    if !cli.skip_extract {
        let scripts_dir = find_scripts_dir(&cli.input)?;
        extract::run_utmt(&cli.input, &extract_dir, &scripts_dir)
            .context("Failed to run utmt extraction")?;
    }

    let backgrounds = extract::load_backgrounds(&extract_dir)?;
    let mut texture_cache = textures::TexturePageCache::new(&extract_dir.join("textures"));
    let sprites = cli
        .export_all_sprites
        .then(|| extract::load_sprites(&extract_dir))
        .transpose()?;
    if let Some(sprites) = sprites.as_deref() {
        crop_and_save_sprite_catalog(sprites, &mut texture_cache, &sprite_catalog_dir)?;
    }
    let room_names = resolve_room_names(&cli.rooms, &extract_dir)?;

    for room_name in &room_names {
        println!("Converting room: {room_name}");
        convert_one_room(
            room_name,
            &backgrounds,
            &extract_dir,
            &mut texture_cache,
            &cli.output,
            &textures_dir,
            &tilesets_dir,
            &sprites_dir,
            &tile_objects_dir,
            cli.tile_size,
        )?;
    }

    println!("Done. Converted {} room(s).", room_names.len());
    Ok(())
}

fn crop_and_save_sprite_catalog(
    sprites: &[schema::SpriteDef],
    texture_cache: &mut textures::TexturePageCache,
    sprite_catalog_dir: &Path,
) -> anyhow::Result<()> {
    for sprite in sprites {
        for (frame_idx, frame) in sprite.frames.iter().enumerate() {
            let out_path = sprite_catalog_dir.join(format!("{}_{}.png", sprite.name, frame_idx));
            if out_path.exists() {
                continue;
            }
            let img = texture_cache.crop(
                frame.texture_page_index,
                frame.source_x,
                frame.source_y,
                frame.source_width,
                frame.source_height,
            )?;
            img.save(&out_path)
                .with_context(|| format!("Failed to save sprite catalog frame {out_path:?}"))?;
        }
    }
    Ok(())
}

fn find_scripts_dir(_data_win: &Path) -> anyhow::Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        let candidate = exe_dir.join("scripts");
        if candidate.join("extract_data.csx").exists() {
            return Ok(candidate);
        }
    }

    let cwd_candidate = std::env::current_dir()?.join("scripts");
    if cwd_candidate.join("extract_data.csx").exists() {
        return Ok(cwd_candidate);
    }

    anyhow::bail!(
        "Could not find scripts/extract_data.csx. \
         Ensure the scripts/ directory is in the current working directory or next to the binary."
    )
}

fn resolve_room_names(rooms_arg: &str, extract_dir: &Path) -> anyhow::Result<Vec<String>> {
    if rooms_arg == "all" {
        return extract::list_rooms(extract_dir);
    }
    Ok(rooms_arg.split(',').map(|s| s.trim().to_string()).collect())
}

fn convert_one_room(
    room_name: &str,
    backgrounds: &HashMap<String, schema::BackgroundDef>,
    extract_dir: &Path,
    texture_cache: &mut textures::TexturePageCache,
    output_dir: &Path,
    textures_dir: &Path,
    tilesets_dir: &Path,
    sprites_dir: &Path,
    tile_objects_dir: &Path,
    tile_size: u32,
) -> anyhow::Result<()> {
    let room = extract::load_room(extract_dir, room_name)?;
    let tile_size = convert::detect_room_tile_size(&room, backgrounds, tile_size);

    let mut used_bgs: Vec<String> = Vec::new();
    for tile in &room.tiles {
        if !used_bgs.contains(&tile.background) {
            used_bgs.push(tile.background.clone());
        }
    }
    for layer in &room.gms2_tile_layers {
        if !used_bgs.contains(&layer.background) {
            used_bgs.push(layer.background.clone());
        }
    }

    crop_and_save_textures(&used_bgs, backgrounds, texture_cache, textures_dir)?;

    let (tiled_map, tilesets, sprite_sources, free_tile_sources) =
        convert::convert_room(&room, backgrounds, tile_size)?;

    crop_and_save_cropped_images(&sprite_sources, texture_cache, sprites_dir)?;
    crop_and_save_cropped_images(&free_tile_sources, texture_cache, tile_objects_dir)?;

    for tileset in &tilesets {
        let tsx_path = tilesets_dir.join(format!("{}.tsx", tileset.name));
        export::tsx::write_tsx(tileset, &tsx_path)?;
    }

    let tmx_path = output_dir.join(format!("{room_name}.tmx"));
    export::tmx::write_tmx(&tiled_map, &tmx_path)?;
    Ok(())
}

fn crop_and_save_textures(
    used_bgs: &[String],
    backgrounds: &HashMap<String, schema::BackgroundDef>,
    texture_cache: &mut textures::TexturePageCache,
    textures_dir: &Path,
) -> anyhow::Result<()> {
    for bg_name in used_bgs {
        let texture_path = textures_dir.join(format!("{bg_name}.png"));
        if texture_path.exists() {
            continue;
        }
        let Some(bg_def) = backgrounds.get(bg_name) else {
            continue;
        };
        let img = texture_cache.crop(
            bg_def.texture_page_index,
            bg_def.source_x,
            bg_def.source_y,
            bg_def.source_width,
            bg_def.source_height,
        )?;
        img.save(&texture_path)
            .with_context(|| format!("Failed to save texture {texture_path:?}"))?;
    }
    Ok(())
}

fn crop_and_save_cropped_images(
    image_sources: &[convert::CroppedImageSourceInfo],
    texture_cache: &mut textures::TexturePageCache,
    output_dir: &Path,
) -> anyhow::Result<()> {
    for image_source in image_sources {
        let out_path = output_dir.join(format!("{}.png", image_source.name));
        if out_path.exists() {
            continue;
        }
        let img = texture_cache.crop(
            image_source.texture_page_index,
            image_source.source_x,
            image_source.source_y,
            image_source.source_width,
            image_source.source_height,
        )?;
        img.save(&out_path)
            .with_context(|| format!("Failed to save cropped image {out_path:?}"))?;
    }
    Ok(())
}
