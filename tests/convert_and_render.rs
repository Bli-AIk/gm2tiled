mod common;

use std::collections::HashMap;

use common::{
    TempDir, background, empty_room, enabled_view, game_object, gms2_layer, rgba, tile, write_png,
};
use gm2tiled::convert;
use gm2tiled::model::{Layer, MapObject, TileLayer, TiledMap, Tileset, TilesetRef};
use gm2tiled::render;
use gm2tiled::textures::TexturePageCache;
use image::RgbaImage;

#[test]
fn convert_room_builds_gms1_free_tiles_objects_and_views() {
    let mut room = empty_room();
    room.tiles = vec![
        tile(0, 0, 0, 0, 20, 20, 100, "bg"),
        tile(7, 3, 20, 0, 10, 20, 100, "bg"),
    ];
    room.game_objects.push(game_object("obj_marker", 0, 8, 6));
    room.views.push(enabled_view());

    let backgrounds = HashMap::from([("bg".to_string(), background("bg", 0, 40, 20, 0, 0))]);
    let (map, tilesets, sprite_sources, free_tile_sources) =
        convert::convert_room(&room, &backgrounds, 20).expect("convert room");

    assert_eq!(map.tile_width, 20);
    assert_eq!(tilesets.len(), 3);
    assert_eq!(sprite_sources.len(), 1);
    assert_eq!(free_tile_sources.len(), 1);

    assert!(map.layers.iter().any(|layer| matches!(layer, Layer::Tile(TileLayer { data, .. }) if data.iter().any(|gid| *gid != 0))));
    assert!(map.layers.iter().any(|layer| {
        matches!(layer, Layer::Object(obj) if obj.objects.iter().any(|object| matches!(object, MapObject::TileObject(_))))
    }));
    assert!(
        map.layers
            .iter()
            .any(|layer| matches!(layer, Layer::Object(obj) if obj.name == "objects"))
    );
    assert!(
        map.layers
            .iter()
            .any(|layer| matches!(layer, Layer::Object(obj) if obj.name == "views"))
    );

    let objects_layer = map
        .layers
        .iter()
        .find_map(|layer| match layer {
            Layer::Object(obj) if obj.name == "objects" => Some(obj),
            _ => None,
        })
        .expect("objects layer");
    let instance = objects_layer
        .objects
        .iter()
        .find_map(|object| match object {
            MapObject::Instance(instance) => Some(instance),
            _ => None,
        })
        .expect("instance object");
    assert!(instance.gid.is_some());
    assert_eq!(instance.x, 10.0);
    assert_eq!(instance.y, 21.0);
}

#[test]
fn convert_room_preserves_gms2_transform_flags_and_mixed_sizes() {
    let mut room = empty_room();
    room.gms2_tile_layers.push(gms2_layer(
        "main",
        0,
        "bg20",
        vec![vec![1 | gm2tiled::tile_flags::GMS2_TILE_MIRROR_FLAG]],
    ));
    room.gms2_tile_layers.push(gms2_layer(
        "fine",
        -10,
        "bg10",
        vec![vec![1 | gm2tiled::tile_flags::GMS2_TILE_ROTATE_FLAG]],
    ));

    let backgrounds = HashMap::from([
        ("bg20".to_string(), background("bg20", 0, 40, 20, 20, 20)),
        ("bg10".to_string(), background("bg10", 0, 20, 10, 10, 10)),
    ]);
    let (map, _tilesets, _sprite_sources, _free_tile_sources) =
        convert::convert_room(&room, &backgrounds, 20).expect("convert room");

    let main_layer = map
        .layers
        .iter()
        .find_map(|layer| match layer {
            Layer::Tile(tile_layer) if tile_layer.name == "main" => Some(tile_layer),
            _ => None,
        })
        .expect("main tile layer");
    assert_eq!(main_layer.data[0], 0x8000_0002);

    let fine_layer = map
        .layers
        .iter()
        .find_map(|layer| match layer {
            Layer::Object(obj) if obj.name == "fine_objects" => Some(obj),
            _ => None,
        })
        .expect("fine object layer");
    let tile_object = fine_layer
        .objects
        .iter()
        .find_map(|object| match object {
            MapObject::TileObject(tile_object) => Some(tile_object),
            _ => None,
        })
        .expect("tile object");
    assert_eq!(tile_object.gid, 0xA000_0004);
}

#[test]
fn render_pipeline_matches_reference_for_transformed_gms2_tiles() {
    let dir = TempDir::new("render_pipeline");
    let textures_dir = dir.path().join("textures");
    std::fs::create_dir_all(&textures_dir).expect("create textures dir");

    let mut page = RgbaImage::new(4, 2);
    page.put_pixel(2, 0, rgba(255, 0, 0, 255));
    page.put_pixel(3, 0, rgba(0, 255, 0, 255));
    page.put_pixel(2, 1, rgba(0, 0, 255, 255));
    page.put_pixel(3, 1, rgba(255, 255, 0, 255));
    write_png(&textures_dir.join("0.png"), &page);

    let backgrounds = HashMap::from([("bg".to_string(), background("bg", 0, 4, 2, 2, 2))]);

    let mut room = empty_room();
    room.width = 2;
    room.height = 2;
    room.gms2_tile_layers.push(gms2_layer(
        "main",
        0,
        "bg",
        vec![vec![1 | gm2tiled::tile_flags::GMS2_TILE_MIRROR_FLAG]],
    ));

    let mut texture_cache = TexturePageCache::new(&textures_dir);
    let mut region_cache = render::RegionCache::new();
    let (reference, stats) = render::render_reference_room_static(
        &room,
        &backgrounds,
        &mut texture_cache,
        &mut region_cache,
    )
    .expect("render reference");
    assert_eq!(stats.gms2_flagged_tiles, 1);
    assert_eq!(reference.get_pixel(0, 0), &rgba(0, 255, 0, 255));
    assert_eq!(reference.get_pixel(1, 0), &rgba(255, 0, 0, 255));
    assert_eq!(reference.get_pixel(0, 1), &rgba(255, 255, 0, 255));
    assert_eq!(reference.get_pixel(1, 1), &rgba(0, 0, 255, 255));

    let (map, tilesets, _, _) = convert::convert_room(&room, &backgrounds, 2).expect("convert");
    let rendered = render::render_tiled_map_static(
        &map,
        &tilesets,
        room.width,
        room.height,
        &mut texture_cache,
        &mut region_cache,
    )
    .expect("render tiled");
    let diff = render::compare_images(&reference, &rendered);
    assert_eq!(diff.metrics.ae_pixels, 0);
}

#[test]
fn render_tiled_map_applies_tiled_flip_flags() {
    let dir = TempDir::new("render_tiled_flags");
    let textures_dir = dir.path().join("textures");
    std::fs::create_dir_all(&textures_dir).expect("create textures dir");

    let mut page = RgbaImage::new(2, 2);
    page.put_pixel(0, 0, rgba(1, 2, 3, 255));
    page.put_pixel(1, 0, rgba(4, 5, 6, 255));
    page.put_pixel(0, 1, rgba(7, 8, 9, 255));
    page.put_pixel(1, 1, rgba(10, 11, 12, 255));
    write_png(&textures_dir.join("0.png"), &page);

    let map = TiledMap {
        width_tiles: 1,
        height_tiles: 1,
        tile_width: 2,
        tile_height: 2,
        background_color: None,
        tilesets: vec![TilesetRef {
            first_gid: 1,
            tsx_path: "tilesets/single.tsx".to_string(),
        }],
        layers: vec![Layer::Tile(TileLayer {
            id: 1,
            name: "base".to_string(),
            width: 1,
            height: 1,
            data: vec![1 | 0x2000_0000],
        })],
        next_layer_id: 2,
        next_object_id: 1,
        speed: 30,
    };
    let tilesets = vec![Tileset {
        name: "single".to_string(),
        tile_width: 2,
        tile_height: 2,
        image_path: "../textures/0.png".to_string(),
        image_width: 2,
        image_height: 2,
        columns: 1,
        tile_count: 1,
        source_texture_page_index: 0,
        source_x: 0,
        source_y: 0,
    }];

    let mut texture_cache = TexturePageCache::new(&textures_dir);
    let mut region_cache = render::RegionCache::new();
    let rendered = render::render_tiled_map_static(
        &map,
        &tilesets,
        2,
        2,
        &mut texture_cache,
        &mut region_cache,
    )
    .expect("render tiled");

    assert_eq!(rendered.get_pixel(0, 0), &rgba(1, 2, 3, 255));
    assert_eq!(rendered.get_pixel(1, 0), &rgba(7, 8, 9, 255));
    assert_eq!(rendered.get_pixel(0, 1), &rgba(4, 5, 6, 255));
    assert_eq!(rendered.get_pixel(1, 1), &rgba(10, 11, 12, 255));
}

#[test]
fn compare_images_reports_size_mismatch_and_changed_pixels() {
    let mut left = RgbaImage::new(1, 1);
    left.put_pixel(0, 0, rgba(1, 2, 3, 255));
    let mut right = RgbaImage::new(2, 1);
    right.put_pixel(0, 0, rgba(9, 9, 9, 255));
    right.put_pixel(1, 0, rgba(1, 1, 1, 255));

    let diff = render::compare_images(&left, &right);
    assert!(!diff.metrics.size_match);
    assert_eq!(diff.metrics.ae_pixels, 2);
    assert!(diff.metrics.rmse > 0.0);
}
