#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use gm2tiled::schema::{
    BackgroundDef, GameObjectData, Gms2TileLayer, RoomData, TileData, ViewData,
};
use image::{Rgba, RgbaImage};

pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(label: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "gm2tiled_{label}_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn write_png(path: &Path, image: &RgbaImage) {
    image.save(path).expect("save png");
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba<u8> {
    Rgba([r, g, b, a])
}

pub fn background(
    name: &str,
    texture_page_index: usize,
    source_width: u32,
    source_height: u32,
    gms2_tile_width: u32,
    gms2_tile_height: u32,
) -> BackgroundDef {
    background_with_layout(
        name,
        texture_page_index,
        source_width,
        source_height,
        gms2_tile_width,
        gms2_tile_height,
        0,
        0,
        0,
        0,
    )
}

pub fn background_with_layout(
    name: &str,
    texture_page_index: usize,
    source_width: u32,
    source_height: u32,
    gms2_tile_width: u32,
    gms2_tile_height: u32,
    gms2_output_border_x: u32,
    gms2_output_border_y: u32,
    gms2_tile_columns: u32,
    gms2_tile_count: u32,
) -> BackgroundDef {
    BackgroundDef {
        name: name.to_string(),
        texture_page_index,
        source_x: 0,
        source_y: 0,
        source_width,
        source_height,
        gms2_tile_width,
        gms2_tile_height,
        gms2_output_border_x,
        gms2_output_border_y,
        gms2_tile_columns,
        gms2_tile_count,
    }
}

pub fn empty_room() -> RoomData {
    RoomData {
        width: 40,
        height: 40,
        speed: 30,
        background_color: 0xFF000000,
        draw_background_color: true,
        tiles: Vec::new(),
        game_objects: Vec::new(),
        views: Vec::new(),
        gms2_tile_layers: Vec::new(),
    }
}

pub fn tile(
    x: i32,
    y: i32,
    source_x: u32,
    source_y: u32,
    width: u32,
    height: u32,
    depth: i32,
    background: &str,
) -> TileData {
    TileData {
        x,
        y,
        source_x,
        source_y,
        width,
        height,
        depth,
        background: background.to_string(),
    }
}

pub fn gms2_layer(
    name: &str,
    depth: i32,
    background: &str,
    tile_data: Vec<Vec<u32>>,
) -> Gms2TileLayer {
    let tiles_y = tile_data.len() as u32;
    let tiles_x = tile_data.first().map_or(0, |row| row.len() as u32);
    Gms2TileLayer {
        name: name.to_string(),
        depth,
        background: background.to_string(),
        tiles_x,
        tiles_y,
        tile_data,
    }
}

pub fn game_object(name: &str, sprite_page: i32, sprite_w: u32, sprite_h: u32) -> GameObjectData {
    GameObjectData {
        x: 12,
        y: 18,
        object_name: name.to_string(),
        instance_id: 99,
        sprite_page,
        sprite_source_x: 0,
        sprite_source_y: 0,
        sprite_source_width: sprite_w,
        sprite_source_height: sprite_h,
        sprite_origin_x: 2,
        sprite_origin_y: 3,
    }
}

pub fn enabled_view() -> ViewData {
    ViewData {
        enabled: true,
        view_x: 1,
        view_y: 2,
        view_width: 30,
        view_height: 20,
        port_x: 3,
        port_y: 4,
        port_width: 60,
        port_height: 40,
    }
}
