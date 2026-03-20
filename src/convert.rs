use std::collections::HashMap;

use anyhow::Context;

use crate::model::{
    InstanceObject, Layer, MapObject, ObjectLayer, TileLayer, TileObjectData, TiledMap, Tileset,
    TilesetRef, ViewObject,
};
use crate::schema::{BackgroundDef, Gms2TileLayer, RoomData, TileData};

struct TilesetInfo {
    first_gid: u32,
    tile_width: u32,
    tile_height: u32,
    columns: u32,
    tile_count: u32,
    source_width: u32,
    source_height: u32,
    name: String,
}

struct SpriteTilesetInfo {
    first_gid: u32,
    source_width: u32,
    source_height: u32,
    texture_page_index: usize,
    source_x: u32,
    source_y: u32,
    origin_x: i32,
    origin_y: i32,
    name: String,
}

pub struct SpriteSourceInfo {
    pub name: String,
    pub texture_page_index: usize,
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: u32,
    pub source_height: u32,
}

pub fn convert_room(
    room: &RoomData,
    backgrounds: &HashMap<String, BackgroundDef>,
    tile_size: u32,
) -> anyhow::Result<(TiledMap, Vec<Tileset>, Vec<SpriteSourceInfo>)> {
    let tileset_map = build_tileset_map(room, backgrounds, tile_size)?;
    let bg_next_gid = tileset_map
        .values()
        .map(|t| t.first_gid + t.tile_count.max(1))
        .max()
        .unwrap_or(1);
    let sprite_tileset_map = build_sprite_tileset_map(room, bg_next_gid);

    let map_w = room.width.div_ceil(tile_size);
    let map_h = room.height.div_ceil(tile_size);

    let mut layer_id: u32 = 1;
    let mut obj_id: u32 = 1;
    let mut layers = Vec::new();

    layers.extend(build_gms1_layers(
        room,
        &tileset_map,
        map_w,
        map_h,
        tile_size,
        &mut layer_id,
        &mut obj_id,
    ));
    layers.extend(build_gms2_layers(room, &tileset_map, &mut layer_id)?);

    if let Some(l) = build_objects_layer(room, &sprite_tileset_map, &mut layer_id, &mut obj_id) {
        layers.push(l);
    }
    if let Some(l) = build_views_layer(room, &mut layer_id, &mut obj_id) {
        layers.push(l);
    }

    let (tilesets_vec, tileset_refs, sprite_sources) =
        build_tileset_lists(&tileset_map, &sprite_tileset_map);
    let background_color = room
        .draw_background_color
        .then(|| argb_to_hex(room.background_color));

    let tiled_map = TiledMap {
        width_tiles: map_w,
        height_tiles: map_h,
        tile_width: tile_size,
        tile_height: tile_size,
        background_color,
        tilesets: tileset_refs,
        layers,
        next_layer_id: layer_id,
        next_object_id: obj_id,
        speed: room.speed,
    };

    Ok((tiled_map, tilesets_vec, sprite_sources))
}

fn build_tileset_lists(
    tileset_map: &HashMap<String, TilesetInfo>,
    sprite_tileset_map: &HashMap<String, SpriteTilesetInfo>,
) -> (Vec<Tileset>, Vec<TilesetRef>, Vec<SpriteSourceInfo>) {
    let mut sorted_bg: Vec<&TilesetInfo> = tileset_map.values().collect();
    sorted_bg.sort_by_key(|t| t.first_gid);

    let mut sorted_spr: Vec<&SpriteTilesetInfo> = sprite_tileset_map.values().collect();
    sorted_spr.sort_by_key(|t| t.first_gid);

    let mut tilesets = Vec::new();
    let mut refs = Vec::new();
    let mut sprite_sources = Vec::new();

    for info in sorted_bg {
        refs.push(TilesetRef {
            first_gid: info.first_gid,
            tsx_path: format!("tilesets/{}.tsx", info.name),
        });
        tilesets.push(Tileset {
            name: info.name.clone(),
            tile_width: info.tile_width,
            tile_height: info.tile_height,
            image_path: format!("../textures/{}.png", info.name),
            image_width: info.source_width,
            image_height: info.source_height,
            columns: info.columns,
            tile_count: info.tile_count,
        });
    }

    for info in sorted_spr {
        refs.push(TilesetRef {
            first_gid: info.first_gid,
            tsx_path: format!("tilesets/{}.tsx", info.name),
        });
        tilesets.push(Tileset {
            name: info.name.clone(),
            tile_width: info.source_width,
            tile_height: info.source_height,
            image_path: format!("../sprites/{}.png", info.name),
            image_width: info.source_width,
            image_height: info.source_height,
            columns: 1,
            tile_count: 1,
        });
        sprite_sources.push(SpriteSourceInfo {
            name: info.name.clone(),
            texture_page_index: info.texture_page_index,
            source_x: info.source_x,
            source_y: info.source_y,
            source_width: info.source_width,
            source_height: info.source_height,
        });
    }

    refs.sort_by_key(|r| r.first_gid);
    (tilesets, refs, sprite_sources)
}

fn build_tileset_map(
    room: &RoomData,
    backgrounds: &HashMap<String, BackgroundDef>,
    tile_size: u32,
) -> anyhow::Result<HashMap<String, TilesetInfo>> {
    let used = collect_used_backgrounds(room);
    let mut tileset_map = HashMap::new();
    let mut next_gid: u32 = 1;

    for bg_name in &used {
        let bg_def = backgrounds
            .get(bg_name)
            .with_context(|| format!("Background '{bg_name}' not found in backgrounds.json"))?;

        let (tile_w, tile_h) = determine_tile_dims(bg_name, room, bg_def, tile_size);
        let columns = bg_def.source_width / tile_w.max(1);
        let rows = bg_def.source_height / tile_h.max(1);
        let tile_count = columns * rows;

        tileset_map.insert(
            bg_name.clone(),
            TilesetInfo {
                first_gid: next_gid,
                tile_width: tile_w,
                tile_height: tile_h,
                columns,
                tile_count,
                source_width: bg_def.source_width,
                source_height: bg_def.source_height,
                name: bg_name.clone(),
            },
        );

        next_gid += tile_count.max(1);
    }

    Ok(tileset_map)
}

fn build_sprite_tileset_map(
    room: &RoomData,
    mut next_gid: u32,
) -> HashMap<String, SpriteTilesetInfo> {
    let mut sprite_map = HashMap::new();
    for obj in &room.game_objects {
        if obj.sprite_page < 0
            || obj.sprite_source_width == 0
            || obj.sprite_source_height == 0
            || sprite_map.contains_key(&obj.object_name)
        {
            continue;
        }
        sprite_map.insert(
            obj.object_name.clone(),
            SpriteTilesetInfo {
                first_gid: next_gid,
                source_width: obj.sprite_source_width,
                source_height: obj.sprite_source_height,
                texture_page_index: obj.sprite_page as usize,
                source_x: obj.sprite_source_x,
                source_y: obj.sprite_source_y,
                origin_x: obj.sprite_origin_x,
                origin_y: obj.sprite_origin_y,
                name: obj.object_name.clone(),
            },
        );
        next_gid += 1;
    }
    sprite_map
}

fn collect_used_backgrounds(room: &RoomData) -> Vec<String> {
    let mut used: Vec<String> = Vec::new();
    for tile in &room.tiles {
        if !tile.background.is_empty() && !used.contains(&tile.background) {
            used.push(tile.background.clone());
        }
    }
    for layer in &room.gms2_tile_layers {
        if !layer.background.is_empty() && !used.contains(&layer.background) {
            used.push(layer.background.clone());
        }
    }
    used
}

fn determine_tile_dims(
    bg_name: &str,
    room: &RoomData,
    bg_def: &BackgroundDef,
    tile_size: u32,
) -> (u32, u32) {
    // Only trust GMS2 tileset dimensions when the room actually uses the
    // background through a GMS2 tile layer. Some legacy Undertale backgrounds
    // carry 32x32 metadata even though the room tiles referencing them are 20x20.
    let used_by_gms2_layer = room
        .gms2_tile_layers
        .iter()
        .any(|layer| layer.background == bg_name);
    if used_by_gms2_layer && bg_def.gms2_tile_width > 0 && bg_def.gms2_tile_height > 0 {
        return (bg_def.gms2_tile_width, bg_def.gms2_tile_height);
    }
    determine_tile_dims_gms1(bg_name, room, tile_size)
}

fn determine_tile_dims_gms1(bg_name: &str, room: &RoomData, tile_size: u32) -> (u32, u32) {
    let mut counts: HashMap<(u32, u32), usize> = HashMap::new();
    for tile in room.tiles.iter().filter(|t| t.background == bg_name) {
        *counts.entry((tile.width, tile.height)).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(dims, _)| dims)
        .unwrap_or((tile_size, tile_size))
}

fn is_grid_aligned(tile: &TileData, tile_size: u32) -> bool {
    tile.width == tile_size
        && tile.height == tile_size
        && tile.x >= 0
        && tile.y >= 0
        && (tile.x as u32).is_multiple_of(tile_size)
        && (tile.y as u32).is_multiple_of(tile_size)
}

fn build_gms1_layers(
    room: &RoomData,
    tileset_map: &HashMap<String, TilesetInfo>,
    map_w: u32,
    map_h: u32,
    tile_size: u32,
    layer_id: &mut u32,
    obj_id: &mut u32,
) -> Vec<Layer> {
    let mut depth_groups: HashMap<i32, Vec<&TileData>> = HashMap::new();
    for tile in &room.tiles {
        depth_groups.entry(tile.depth).or_default().push(tile);
    }
    let mut depths: Vec<i32> = depth_groups.keys().copied().collect();
    depths.sort_by(|a, b| b.cmp(a));

    let mut layers = Vec::new();
    for depth in depths {
        let group = &depth_groups[&depth];
        let grid: Vec<&TileData> = group
            .iter()
            .filter(|t| is_grid_aligned(t, tile_size))
            .copied()
            .collect();
        let free: Vec<&TileData> = group
            .iter()
            .filter(|t| !is_grid_aligned(t, tile_size))
            .copied()
            .collect();

        if !grid.is_empty() {
            let tl =
                build_tile_layer_gms1(&grid, tileset_map, map_w, map_h, tile_size, depth, layer_id);
            layers.push(Layer::Tile(tl));
        }
        if !free.is_empty() {
            let ol = build_object_layer_tiles(&free, tileset_map, depth, layer_id, obj_id);
            layers.push(Layer::Object(ol));
        }
    }
    layers
}

fn build_tile_layer_gms1(
    tiles: &[&TileData],
    tileset_map: &HashMap<String, TilesetInfo>,
    map_w: u32,
    map_h: u32,
    tile_size: u32,
    depth: i32,
    layer_id: &mut u32,
) -> TileLayer {
    let mut data = vec![0u32; (map_w * map_h) as usize];
    for tile in tiles {
        let Some(info) = tileset_map.get(&tile.background) else {
            continue;
        };
        let col_in_ts = tile.source_x / info.tile_width.max(1);
        let row_in_ts = tile.source_y / info.tile_height.max(1);
        let local_id = row_in_ts * info.columns + col_in_ts;
        let gid = info.first_gid + local_id;
        let map_col = tile.x as u32 / tile_size;
        let map_row = tile.y as u32 / tile_size;
        if map_col < map_w && map_row < map_h {
            data[(map_row * map_w + map_col) as usize] = gid;
        }
    }
    let id = *layer_id;
    *layer_id += 1;
    TileLayer {
        id,
        name: format!("depth_{depth}"),
        width: map_w,
        height: map_h,
        data,
    }
}

fn build_object_layer_tiles(
    tiles: &[&TileData],
    tileset_map: &HashMap<String, TilesetInfo>,
    depth: i32,
    layer_id: &mut u32,
    obj_id: &mut u32,
) -> ObjectLayer {
    let mut objects = Vec::new();
    for tile in tiles {
        let Some(info) = tileset_map.get(&tile.background) else {
            continue;
        };
        let col_in_ts = tile.source_x / info.tile_width.max(1);
        let row_in_ts = tile.source_y / info.tile_height.max(1);
        let local_id = row_in_ts * info.columns + col_in_ts;
        let gid = info.first_gid + local_id;
        let id = *obj_id;
        *obj_id += 1;
        objects.push(MapObject::TileObject(TileObjectData {
            id,
            gid,
            x: tile.x as f32,
            y: tile.y as f32 + tile.height as f32,
            width: tile.width as f32,
            height: tile.height as f32,
        }));
    }
    let id = *layer_id;
    *layer_id += 1;
    ObjectLayer {
        id,
        name: format!("depth_{depth}_tiles"),
        objects,
    }
}

fn build_gms2_layers(
    room: &RoomData,
    tileset_map: &HashMap<String, TilesetInfo>,
    layer_id: &mut u32,
) -> anyhow::Result<Vec<Layer>> {
    let mut sorted: Vec<&Gms2TileLayer> = room.gms2_tile_layers.iter().collect();
    sorted.sort_by(|a, b| b.depth.cmp(&a.depth));

    let mut layers = Vec::new();
    for gms2_layer in sorted {
        if gms2_layer.background.is_empty() {
            continue;
        }
        let info = tileset_map
            .get(&gms2_layer.background)
            .with_context(|| format!("Tileset '{}' not found", gms2_layer.background))?;
        let tl = build_gms2_tile_layer(gms2_layer, info, layer_id);
        layers.push(Layer::Tile(tl));
    }
    Ok(layers)
}

fn build_gms2_tile_layer(
    gms2_layer: &Gms2TileLayer,
    info: &TilesetInfo,
    layer_id: &mut u32,
) -> TileLayer {
    let w = gms2_layer.tiles_x;
    let h = gms2_layer.tiles_y;
    let mut data = vec![0u32; (w * h) as usize];

    for (row_idx, row) in gms2_layer.tile_data.iter().enumerate() {
        for (col_idx, &raw) in row.iter().enumerate() {
            if raw == 0 {
                continue;
            }
            let tile_idx = raw & 0x7_FFFF;
            let gid = info.first_gid + tile_idx;
            let pos = row_idx as u32 * w + col_idx as u32;
            if (pos as usize) < data.len() {
                data[pos as usize] = gid;
            }
        }
    }

    let id = *layer_id;
    *layer_id += 1;
    TileLayer {
        id,
        name: gms2_layer.name.clone(),
        width: w,
        height: h,
        data,
    }
}

fn build_objects_layer(
    room: &RoomData,
    sprite_tileset_map: &HashMap<String, SpriteTilesetInfo>,
    layer_id: &mut u32,
    obj_id: &mut u32,
) -> Option<Layer> {
    if room.game_objects.is_empty() {
        return None;
    }
    let mut objects = Vec::new();
    for obj in &room.game_objects {
        let id = *obj_id;
        *obj_id += 1;
        let (gid, x, y, width, height) = match sprite_tileset_map.get(&obj.object_name) {
            Some(spr) => {
                let w = spr.source_width as f32;
                let h = spr.source_height as f32;
                let tile_x = obj.x as f32 - spr.origin_x as f32;
                let tile_y = obj.y as f32 - spr.origin_y as f32 + h;
                (Some(spr.first_gid), tile_x, tile_y, w, h)
            }
            None => (None, obj.x as f32, obj.y as f32, 0.0, 0.0),
        };
        objects.push(MapObject::Instance(InstanceObject {
            id,
            obj_type: obj.object_name.clone(),
            x,
            y,
            width,
            height,
            instance_id: obj.instance_id,
            gid,
        }));
    }
    let id = *layer_id;
    *layer_id += 1;
    Some(Layer::Object(ObjectLayer {
        id,
        name: "objects".to_string(),
        objects,
    }))
}

fn build_views_layer(room: &RoomData, layer_id: &mut u32, obj_id: &mut u32) -> Option<Layer> {
    let enabled: Vec<_> = room.views.iter().filter(|v| v.enabled).collect();
    if enabled.is_empty() {
        return None;
    }
    let mut objects = Vec::new();
    for view in enabled {
        let id = *obj_id;
        *obj_id += 1;
        objects.push(MapObject::View(ViewObject {
            id,
            x: view.view_x as f32,
            y: view.view_y as f32,
            width: view.view_width as f32,
            height: view.view_height as f32,
            port_x: view.port_x,
            port_y: view.port_y,
            port_width: view.port_width,
            port_height: view.port_height,
        }));
    }
    let id = *layer_id;
    *layer_id += 1;
    Some(Layer::Object(ObjectLayer {
        id,
        name: "views".to_string(),
        objects,
    }))
}

fn argb_to_hex(color: u32) -> String {
    let r = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = color & 0xFF;
    format!("#{r:02x}{g:02x}{b:02x}")
}
