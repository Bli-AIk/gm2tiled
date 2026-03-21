pub const GMS2_TILE_INDEX_MASK: u32 = 0x0007_FFFF;
pub const GMS2_TILE_MIRROR_FLAG: u32 = 0x1000_0000;
pub const GMS2_TILE_FLIP_FLAG: u32 = 0x2000_0000;
pub const GMS2_TILE_ROTATE_FLAG: u32 = 0x4000_0000;
pub const GMS2_TILE_FLAG_MASK: u32 =
    GMS2_TILE_MIRROR_FLAG | GMS2_TILE_FLIP_FLAG | GMS2_TILE_ROTATE_FLAG;

const TILED_FLIP_HORIZONTAL_FLAG: u32 = 0x8000_0000;
const TILED_FLIP_VERTICAL_FLAG: u32 = 0x4000_0000;
const TILED_FLIP_DIAGONAL_FLAG: u32 = 0x2000_0000;

pub fn gms2_tile_index(raw: u32) -> u32 {
    raw & GMS2_TILE_INDEX_MASK
}

pub fn gms2_tile_flags(raw: u32) -> u32 {
    raw & GMS2_TILE_FLAG_MASK
}

pub fn gms2_raw_to_tiled_gid(raw: u32, first_gid: u32) -> u32 {
    (first_gid + gms2_tile_index(raw)) | gms2_raw_to_tiled_transform_flags(raw)
}

pub fn gms2_raw_to_tiled_transform_flags(raw: u32) -> u32 {
    gms2_flags_to_tiled_flags(gms2_tile_flags(raw))
}

fn gms2_flags_to_tiled_flags(flags: u32) -> u32 {
    let mirror = flags & GMS2_TILE_MIRROR_FLAG != 0;
    let flip = flags & GMS2_TILE_FLIP_FLAG != 0;
    let rotate = flags & GMS2_TILE_ROTATE_FLAG != 0;

    match (mirror, flip, rotate) {
        (false, false, false) => 0,
        (true, false, false) => TILED_FLIP_HORIZONTAL_FLAG,
        (false, true, false) => TILED_FLIP_VERTICAL_FLAG,
        (true, true, false) => TILED_FLIP_HORIZONTAL_FLAG | TILED_FLIP_VERTICAL_FLAG,
        // GameMaker stores mirror/flip as axis flips and a separate clockwise 90-degree bit.
        // We map those to Tiled by treating mirror/flip as local-axis flips applied before
        // the final 90-degree rotation, which matches the engine's sprite-style transform stack.
        (false, false, true) => TILED_FLIP_DIAGONAL_FLAG | TILED_FLIP_HORIZONTAL_FLAG,
        (true, false, true) => {
            TILED_FLIP_DIAGONAL_FLAG | TILED_FLIP_HORIZONTAL_FLAG | TILED_FLIP_VERTICAL_FLAG
        }
        (false, true, true) => TILED_FLIP_DIAGONAL_FLAG,
        (true, true, true) => TILED_FLIP_DIAGONAL_FLAG | TILED_FLIP_VERTICAL_FLAG,
    }
}
