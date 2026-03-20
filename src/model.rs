pub struct TiledMap {
    pub width_tiles: u32,
    pub height_tiles: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub background_color: Option<String>,
    pub tilesets: Vec<TilesetRef>,
    pub layers: Vec<Layer>,
    pub next_layer_id: u32,
    pub next_object_id: u32,
    pub speed: u32,
}

pub struct TilesetRef {
    pub first_gid: u32,
    pub tsx_path: String,
}

pub enum Layer {
    Tile(TileLayer),
    Object(ObjectLayer),
}

pub struct TileLayer {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u32>,
}

pub struct ObjectLayer {
    pub id: u32,
    pub name: String,
    pub objects: Vec<MapObject>,
}

pub enum MapObject {
    Instance(InstanceObject),
    TileObject(TileObjectData),
    View(ViewObject),
}

pub struct InstanceObject {
    pub id: u32,
    pub obj_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub instance_id: u32,
    pub gid: Option<u32>,
}

pub struct TileObjectData {
    pub id: u32,
    pub gid: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct ViewObject {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub port_x: i32,
    pub port_y: i32,
    pub port_width: u32,
    pub port_height: u32,
}

pub struct Tileset {
    pub name: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub image_path: String,
    pub image_width: u32,
    pub image_height: u32,
    pub columns: u32,
    pub tile_count: u32,
    #[allow(dead_code)]
    pub source_texture_page_index: usize,
    #[allow(dead_code)]
    pub source_x: u32,
    #[allow(dead_code)]
    pub source_y: u32,
}
