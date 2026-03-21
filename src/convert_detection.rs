use std::collections::HashMap;

use crate::schema::{BackgroundDef, RoomData};

pub fn detect_room_tile_size(
    room: &RoomData,
    backgrounds: &HashMap<String, BackgroundDef>,
    fallback_tile_size: u32,
) -> u32 {
    detect_gms2_room_tile_size(room, backgrounds)
        .or_else(|| detect_gms1_room_tile_size(room))
        .unwrap_or_else(|| fallback_tile_size.max(1))
}

fn detect_gms2_room_tile_size(
    room: &RoomData,
    backgrounds: &HashMap<String, BackgroundDef>,
) -> Option<u32> {
    let mut counts: HashMap<(u32, u32), u64> = HashMap::new();

    for layer in &room.gms2_tile_layers {
        let Some(background) = backgrounds.get(&layer.background) else {
            continue;
        };
        if background.gms2_tile_width == 0 || background.gms2_tile_height == 0 {
            continue;
        }
        let non_zero_tiles = layer
            .tile_data
            .iter()
            .flatten()
            .filter(|&&raw| raw != 0)
            .count() as u64;
        *counts
            .entry((background.gms2_tile_width, background.gms2_tile_height))
            .or_default() += non_zero_tiles.max(1);
    }

    counts
        .into_iter()
        .max_by_key(|((width, height), count)| (*count, *width == *height, *width))
        .and_then(|((width, height), _)| (width == height).then_some(width.max(1)))
}

fn detect_gms1_room_tile_size(room: &RoomData) -> Option<u32> {
    let mut candidates: Vec<u32> = room
        .tiles
        .iter()
        .flat_map(|tile| [tile.width, tile.height])
        .collect();
    candidates.sort_unstable();
    candidates.dedup();

    let mut best: Option<(u32, usize, usize)> = None;
    for candidate in candidates {
        if candidate == 0 {
            continue;
        }

        let mut exact = 0usize;
        let mut aligned = 0usize;
        for tile in &room.tiles {
            if tile.x < 0
                || tile.y < 0
                || !(tile.x as u32).is_multiple_of(candidate)
                || !(tile.y as u32).is_multiple_of(candidate)
            {
                continue;
            }
            if tile.width == candidate && tile.height == candidate {
                exact += 1;
                aligned += 1;
            } else if tile.width.is_multiple_of(candidate) && tile.height.is_multiple_of(candidate)
            {
                aligned += 1;
            }
        }

        if aligned == 0 {
            continue;
        }

        match best {
            Some((best_candidate, best_exact, best_aligned))
                if (exact, aligned, std::cmp::Reverse(candidate))
                    <= (best_exact, best_aligned, std::cmp::Reverse(best_candidate)) => {}
            _ => best = Some((candidate, exact, aligned)),
        }
    }

    best.map(|(candidate, _, _)| candidate)
}
