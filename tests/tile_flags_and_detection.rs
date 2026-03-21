mod common;

use std::collections::HashMap;

use common::{background, empty_room, gms2_layer, tile};
use gm2tiled::{convert, tile_flags};

#[test]
fn gms2_flags_map_to_expected_tiled_bits() {
    let idx = 0x35;
    let first_gid = 7;
    let cases = [
        (0, 0),
        (tile_flags::GMS2_TILE_MIRROR_FLAG, 0x8000_0000),
        (tile_flags::GMS2_TILE_FLIP_FLAG, 0x4000_0000),
        (
            tile_flags::GMS2_TILE_MIRROR_FLAG | tile_flags::GMS2_TILE_FLIP_FLAG,
            0xC000_0000,
        ),
        (tile_flags::GMS2_TILE_ROTATE_FLAG, 0xA000_0000),
        (
            tile_flags::GMS2_TILE_ROTATE_FLAG | tile_flags::GMS2_TILE_MIRROR_FLAG,
            0xE000_0000,
        ),
        (
            tile_flags::GMS2_TILE_ROTATE_FLAG | tile_flags::GMS2_TILE_FLIP_FLAG,
            0x2000_0000,
        ),
        (
            tile_flags::GMS2_TILE_ROTATE_FLAG
                | tile_flags::GMS2_TILE_MIRROR_FLAG
                | tile_flags::GMS2_TILE_FLIP_FLAG,
            0x6000_0000,
        ),
    ];

    for (gms2_flags, tiled_flags) in cases {
        let raw = idx | gms2_flags;
        assert_eq!(tile_flags::gms2_tile_index(raw), idx);
        assert_eq!(tile_flags::gms2_tile_flags(raw), gms2_flags);
        assert_eq!(
            tile_flags::gms2_raw_to_tiled_transform_flags(raw),
            tiled_flags
        );
        assert_eq!(
            tile_flags::gms2_raw_to_tiled_gid(raw, first_gid),
            (first_gid + idx) | tiled_flags
        );
    }
}

#[test]
fn detect_room_tile_size_prefers_dominant_square_gms2_tiles() {
    let mut room = empty_room();
    room.gms2_tile_layers.push(gms2_layer(
        "main",
        0,
        "bg40",
        vec![vec![1, 1, 1], vec![1, 0, 1]],
    ));
    room.gms2_tile_layers
        .push(gms2_layer("small", 0, "bg20", vec![vec![1]]));

    let backgrounds = HashMap::from([
        ("bg40".to_string(), background("bg40", 0, 80, 80, 40, 40)),
        ("bg20".to_string(), background("bg20", 0, 40, 40, 20, 20)),
    ]);

    assert_eq!(convert::detect_room_tile_size(&room, &backgrounds, 16), 40);
}

#[test]
fn detect_room_tile_size_uses_aligned_gms1_candidates_before_fallback() {
    let mut room = empty_room();
    room.gms2_tile_layers.clear();
    room.tiles = vec![
        tile(0, 0, 0, 0, 20, 20, 0, "bg"),
        tile(20, 0, 20, 0, 20, 20, 0, "bg"),
        tile(0, 20, 0, 20, 40, 40, 0, "bg"),
        tile(-5, 0, 0, 0, 10, 10, 0, "bg"),
    ];

    assert_eq!(
        convert::detect_room_tile_size(&room, &HashMap::new(), 16),
        20
    );
}

#[test]
fn detect_room_tile_size_falls_back_when_room_has_no_tiles() {
    let room = empty_room();
    assert_eq!(convert::detect_room_tile_size(&room, &HashMap::new(), 0), 1);
}
