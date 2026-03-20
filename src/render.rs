use std::collections::{HashMap, hash_map::Entry};

use anyhow::Context;
use image::{Rgba, RgbaImage};
use serde::Serialize;

use crate::model::{Layer, MapObject, TiledMap, Tileset};
use crate::schema::{BackgroundDef, RoomData};
use crate::textures::TexturePageCache;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct RegionKey {
    texture_page_index: usize,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
}

pub struct RegionCache {
    regions: HashMap<RegionKey, RgbaImage>,
}

impl RegionCache {
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
        }
    }

    fn get(
        &mut self,
        texture_cache: &mut TexturePageCache,
        key: RegionKey,
    ) -> anyhow::Result<&RgbaImage> {
        if let Entry::Vacant(entry) = self.regions.entry(key) {
            let img = texture_cache
                .crop(
                    key.texture_page_index,
                    key.source_x,
                    key.source_y,
                    key.source_width,
                    key.source_height,
                )?
                .to_rgba8();
            entry.insert(img);
        }
        Ok(&self.regions[&key])
    }
}

#[derive(Debug, Default)]
pub struct ReferenceStats {
    pub static_draw_ops: u64,
    pub gms2_flagged_tiles: u64,
    pub instance_count: usize,
    pub enabled_view_count: usize,
}

#[derive(Debug, Serialize)]
pub struct PixelDiffMetrics {
    pub ae_pixels: u64,
    pub ae_percent: f64,
    pub rmse: f64,
    pub size_match: bool,
}

pub struct PixelDiffResult {
    pub diff_image: RgbaImage,
    pub metrics: PixelDiffMetrics,
}

pub fn render_reference_room_static(
    room: &RoomData,
    backgrounds: &HashMap<String, BackgroundDef>,
    texture_cache: &mut TexturePageCache,
    region_cache: &mut RegionCache,
) -> anyhow::Result<(RgbaImage, ReferenceStats)> {
    let mut canvas = new_canvas(room.width, room.height, room_background(room));
    let mut stats = ReferenceStats {
        instance_count: room.game_objects.len(),
        enabled_view_count: room.views.iter().filter(|view| view.enabled).count(),
        ..ReferenceStats::default()
    };

    let mut sorted_tiles: Vec<_> = room.tiles.iter().collect();
    sorted_tiles.sort_by(|a, b| b.depth.cmp(&a.depth));
    for tile in sorted_tiles {
        let Some(background) = backgrounds.get(&tile.background) else {
            continue;
        };
        let key = RegionKey {
            texture_page_index: background.texture_page_index,
            source_x: background.source_x + tile.source_x,
            source_y: background.source_y + tile.source_y,
            source_width: tile.width,
            source_height: tile.height,
        };
        let region = region_cache.get(texture_cache, key)?;
        alpha_blit(&mut canvas, region, tile.x, tile.y);
        stats.static_draw_ops += 1;
    }

    let mut sorted_gms2: Vec<_> = room.gms2_tile_layers.iter().collect();
    sorted_gms2.sort_by(|a, b| b.depth.cmp(&a.depth));
    for layer in sorted_gms2 {
        let Some(background) = backgrounds.get(&layer.background) else {
            continue;
        };
        render_gms2_layer(
            &mut canvas,
            background,
            &layer.tile_data,
            texture_cache,
            region_cache,
            &mut stats,
        )?;
    }

    Ok((canvas, stats))
}

pub fn render_tiled_map_static(
    map: &TiledMap,
    tilesets: &[Tileset],
    room_width: u32,
    room_height: u32,
    texture_cache: &mut TexturePageCache,
    region_cache: &mut RegionCache,
) -> anyhow::Result<RgbaImage> {
    let mut canvas = new_canvas(room_width, room_height, tiled_background(map));
    let render_tilesets = build_render_tilesets(map, tilesets)?;

    for layer in &map.layers {
        match layer {
            Layer::Tile(tile_layer) => render_tiled_tile_layer(
                &mut canvas,
                tile_layer,
                map.tile_width,
                map.tile_height,
                &render_tilesets,
                texture_cache,
                region_cache,
            )?,
            Layer::Object(object_layer) => render_tiled_object_layer(
                &mut canvas,
                object_layer,
                &render_tilesets,
                texture_cache,
                region_cache,
            )?,
        }
    }

    Ok(canvas)
}

pub fn compare_images(reference: &RgbaImage, rendered: &RgbaImage) -> PixelDiffResult {
    let width = reference.width().max(rendered.width());
    let height = reference.height().max(rendered.height());
    let mut diff_image = RgbaImage::new(width, height);

    let size_match = reference.dimensions() == rendered.dimensions();
    let total_pixels = (width as u64 * height as u64).max(1);
    let mut ae_pixels = 0u64;
    let mut sum_squared = 0f64;

    for y in 0..height {
        for x in 0..width {
            let left = pixel_or_transparent(reference, x, y);
            let right = pixel_or_transparent(rendered, x, y);
            if left != right {
                ae_pixels += 1;
            }

            let mut diff_channels = [0u8; 4];
            for channel in 0..4 {
                let delta = left[channel] as f64 - right[channel] as f64;
                sum_squared += delta * delta;
                diff_channels[channel] = left[channel].abs_diff(right[channel]);
            }

            let diff_pixel = if diff_channels.iter().all(|channel| *channel == 0) {
                Rgba([0, 0, 0, 0])
            } else if diff_channels[0] == 0
                && diff_channels[1] == 0
                && diff_channels[2] == 0
                && diff_channels[3] > 0
            {
                Rgba([255, 0, 255, 255])
            } else {
                Rgba([diff_channels[0], diff_channels[1], diff_channels[2], 255])
            };
            diff_image.put_pixel(x, y, diff_pixel);
        }
    }

    let rmse = (sum_squared / (total_pixels as f64 * 4.0 * 255.0 * 255.0)).sqrt();
    let ae_percent = ae_pixels as f64 / total_pixels as f64;

    PixelDiffResult {
        diff_image,
        metrics: PixelDiffMetrics {
            ae_pixels,
            ae_percent,
            rmse,
            size_match,
        },
    }
}

struct RenderTileset {
    first_gid: u32,
    last_gid: u32,
    texture_page_index: usize,
    source_x: u32,
    source_y: u32,
    tile_width: u32,
    tile_height: u32,
    columns: u32,
}

fn build_render_tilesets(
    map: &TiledMap,
    tilesets: &[Tileset],
) -> anyhow::Result<Vec<RenderTileset>> {
    let mut first_gid_by_name = HashMap::new();
    for tileset_ref in &map.tilesets {
        let name = tileset_name_from_path(&tileset_ref.tsx_path);
        first_gid_by_name.insert(name, tileset_ref.first_gid);
    }

    let mut render_tilesets = Vec::new();
    for tileset in tilesets {
        let first_gid = *first_gid_by_name
            .get(tileset.name.as_str())
            .with_context(|| format!("Missing firstgid for tileset '{}'", tileset.name))?;
        let last_gid = first_gid + tileset.tile_count.saturating_sub(1);
        render_tilesets.push(RenderTileset {
            first_gid,
            last_gid,
            texture_page_index: tileset.source_texture_page_index,
            source_x: tileset.source_x,
            source_y: tileset.source_y,
            tile_width: tileset.tile_width.max(1),
            tile_height: tileset.tile_height.max(1),
            columns: tileset.columns.max(1),
        });
    }
    render_tilesets.sort_by_key(|tileset| tileset.first_gid);
    Ok(render_tilesets)
}

fn render_gms2_layer(
    canvas: &mut RgbaImage,
    background: &BackgroundDef,
    tile_data: &[Vec<u32>],
    texture_cache: &mut TexturePageCache,
    region_cache: &mut RegionCache,
    stats: &mut ReferenceStats,
) -> anyhow::Result<()> {
    let tile_width = background.gms2_tile_width.max(1);
    let tile_height = background.gms2_tile_height.max(1);
    let columns = background.source_width / tile_width;
    if columns == 0 {
        return Ok(());
    }

    for (row_idx, row) in tile_data.iter().enumerate() {
        for (col_idx, &raw) in row.iter().enumerate() {
            if raw == 0 {
                continue;
            }
            stats.gms2_flagged_tiles += u64::from(raw & !0x7_FFFF != 0);
            let tile_idx = raw & 0x7_FFFF;
            let src_col = tile_idx % columns;
            let src_row = tile_idx / columns;
            let key = RegionKey {
                texture_page_index: background.texture_page_index,
                source_x: background.source_x + src_col * tile_width,
                source_y: background.source_y + src_row * tile_height,
                source_width: tile_width,
                source_height: tile_height,
            };
            let region = region_cache.get(texture_cache, key)?;
            alpha_blit(
                canvas,
                region,
                col_idx as i32 * tile_width as i32,
                row_idx as i32 * tile_height as i32,
            );
            stats.static_draw_ops += 1;
        }
    }
    Ok(())
}

fn render_tiled_tile_layer(
    canvas: &mut RgbaImage,
    tile_layer: &crate::model::TileLayer,
    map_tile_width: u32,
    map_tile_height: u32,
    render_tilesets: &[RenderTileset],
    texture_cache: &mut TexturePageCache,
    region_cache: &mut RegionCache,
) -> anyhow::Result<()> {
    for row in 0..tile_layer.height {
        for col in 0..tile_layer.width {
            let idx = (row * tile_layer.width + col) as usize;
            let gid = tile_layer.data[idx];
            let Some(tileset) = tileset_for_gid(render_tilesets, gid) else {
                continue;
            };
            let local_id = gid - tileset.first_gid;
            let src_col = local_id % tileset.columns.max(1);
            let src_row = local_id / tileset.columns.max(1);
            let key = RegionKey {
                texture_page_index: tileset.texture_page_index,
                source_x: tileset.source_x + src_col * tileset.tile_width,
                source_y: tileset.source_y + src_row * tileset.tile_height,
                source_width: tileset.tile_width,
                source_height: tileset.tile_height,
            };
            let region = region_cache.get(texture_cache, key)?;
            alpha_blit(
                canvas,
                region,
                col as i32 * map_tile_width as i32,
                row as i32 * map_tile_height as i32,
            );
        }
    }
    Ok(())
}

fn render_tiled_tile_object(
    canvas: &mut RgbaImage,
    tile_object: &crate::model::TileObjectData,
    tileset: &RenderTileset,
    texture_cache: &mut TexturePageCache,
    region_cache: &mut RegionCache,
) -> anyhow::Result<()> {
    let local_id = tile_object.gid - tileset.first_gid;
    let src_col = local_id % tileset.columns.max(1);
    let src_row = local_id / tileset.columns.max(1);
    let key = RegionKey {
        texture_page_index: tileset.texture_page_index,
        source_x: tileset.source_x + src_col * tileset.tile_width,
        source_y: tileset.source_y + src_row * tileset.tile_height,
        source_width: tile_object.width.round() as u32,
        source_height: tile_object.height.round() as u32,
    };
    let region = region_cache.get(texture_cache, key)?;
    alpha_blit(
        canvas,
        region,
        tile_object.x.round() as i32,
        (tile_object.y - tile_object.height).round() as i32,
    );
    Ok(())
}

fn render_tiled_object_layer(
    canvas: &mut RgbaImage,
    object_layer: &crate::model::ObjectLayer,
    render_tilesets: &[RenderTileset],
    texture_cache: &mut TexturePageCache,
    region_cache: &mut RegionCache,
) -> anyhow::Result<()> {
    for object in &object_layer.objects {
        if let MapObject::TileObject(tile_object) = object {
            let Some(tileset) = tileset_for_gid(render_tilesets, tile_object.gid) else {
                continue;
            };
            render_tiled_tile_object(canvas, tile_object, tileset, texture_cache, region_cache)?;
        }
    }
    Ok(())
}

fn tileset_for_gid(render_tilesets: &[RenderTileset], gid: u32) -> Option<&RenderTileset> {
    if gid == 0 {
        return None;
    }
    render_tilesets
        .iter()
        .find(|tileset| gid >= tileset.first_gid && gid <= tileset.last_gid)
}

fn tileset_name_from_path(path: &str) -> &str {
    path.rsplit('/')
        .next()
        .unwrap_or(path)
        .trim_end_matches(".tsx")
}

fn new_canvas(width: u32, height: u32, background: Rgba<u8>) -> RgbaImage {
    RgbaImage::from_pixel(width, height, background)
}

fn room_background(room: &RoomData) -> Rgba<u8> {
    if room.draw_background_color {
        argb_to_rgba(room.background_color)
    } else {
        Rgba([0, 0, 0, 0])
    }
}

fn tiled_background(map: &TiledMap) -> Rgba<u8> {
    map.background_color
        .as_deref()
        .and_then(parse_hex_rgb)
        .unwrap_or(Rgba([0, 0, 0, 0]))
}

fn parse_hex_rgb(value: &str) -> Option<Rgba<u8>> {
    let stripped = value.strip_prefix('#')?;
    if stripped.len() != 6 {
        return None;
    }
    let rgb = u32::from_str_radix(stripped, 16).ok()?;
    Some(Rgba([
        ((rgb >> 16) & 0xFF) as u8,
        ((rgb >> 8) & 0xFF) as u8,
        (rgb & 0xFF) as u8,
        255,
    ]))
}

fn argb_to_rgba(color: u32) -> Rgba<u8> {
    Rgba([
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8,
        ((color >> 24) & 0xFF) as u8,
    ])
}

fn pixel_or_transparent(image: &RgbaImage, x: u32, y: u32) -> Rgba<u8> {
    if x < image.width() && y < image.height() {
        *image.get_pixel(x, y)
    } else {
        Rgba([0, 0, 0, 0])
    }
}

fn alpha_blit(canvas: &mut RgbaImage, source: &RgbaImage, x: i32, y: i32) {
    for src_y in 0..source.height() as i32 {
        for src_x in 0..source.width() as i32 {
            let dst_x = x + src_x;
            let dst_y = y + src_y;
            if dst_x < 0
                || dst_y < 0
                || dst_x >= canvas.width() as i32
                || dst_y >= canvas.height() as i32
            {
                continue;
            }

            let top = *source.get_pixel(src_x as u32, src_y as u32);
            if top[3] == 0 {
                continue;
            }

            let bottom = canvas.get_pixel_mut(dst_x as u32, dst_y as u32);
            *bottom = alpha_over(*bottom, top);
        }
    }
}

fn alpha_over(bottom: Rgba<u8>, top: Rgba<u8>) -> Rgba<u8> {
    let top_alpha = top[3] as f32 / 255.0;
    let bottom_alpha = bottom[3] as f32 / 255.0;
    let out_alpha = top_alpha + bottom_alpha * (1.0 - top_alpha);
    if out_alpha <= f32::EPSILON {
        return Rgba([0, 0, 0, 0]);
    }

    let mut out = [0u8; 4];
    for channel in 0..3 {
        let top_value = top[channel] as f32 / 255.0;
        let bottom_value = bottom[channel] as f32 / 255.0;
        let out_value =
            (top_value * top_alpha + bottom_value * bottom_alpha * (1.0 - top_alpha)) / out_alpha;
        out[channel] = (out_value * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    out[3] = (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    Rgba(out)
}
