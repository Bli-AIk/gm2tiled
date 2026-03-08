use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundDef {
    pub name: String,
    pub texture_page_index: usize,
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: u32,
    pub source_height: u32,
    /// GMS2 tile dimensions stored in the Background resource (0 = not a GMS2 tileset)
    #[serde(default)]
    pub gms2_tile_width: u32,
    #[serde(default)]
    pub gms2_tile_height: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomData {
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub background_color: u32,
    pub draw_background_color: bool,
    #[serde(default)]
    pub tiles: Vec<TileData>,
    #[serde(default)]
    pub game_objects: Vec<GameObjectData>,
    #[serde(default)]
    pub views: Vec<ViewData>,
    #[serde(default)]
    pub gms2_tile_layers: Vec<Gms2TileLayer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileData {
    pub x: i32,
    pub y: i32,
    pub source_x: u32,
    pub source_y: u32,
    pub width: u32,
    pub height: u32,
    pub depth: i32,
    pub background: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameObjectData {
    pub x: i32,
    pub y: i32,
    pub object_name: String,
    pub instance_id: u32,
    #[serde(default = "neg_one_i32")]
    pub sprite_page: i32,
    #[serde(default)]
    pub sprite_source_x: u32,
    #[serde(default)]
    pub sprite_source_y: u32,
    #[serde(default)]
    pub sprite_source_width: u32,
    #[serde(default)]
    pub sprite_source_height: u32,
    #[serde(default)]
    pub sprite_origin_x: i32,
    #[serde(default)]
    pub sprite_origin_y: i32,
}

fn neg_one_i32() -> i32 {
    -1
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewData {
    pub enabled: bool,
    pub view_x: i32,
    pub view_y: i32,
    pub view_width: u32,
    pub view_height: u32,
    pub port_x: i32,
    pub port_y: i32,
    pub port_width: u32,
    pub port_height: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gms2TileLayer {
    pub name: String,
    pub depth: i32,
    pub background: String,
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub tile_data: Vec<Vec<u32>>,
}
