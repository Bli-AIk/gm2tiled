mod common;

use std::fs;

use common::TempDir;
use gm2tiled::export::{tmx, tsx};
use gm2tiled::model::{
    InstanceObject, Layer, MapObject, ObjectLayer, TileLayer, TileObjectData, TiledMap, Tileset,
    TilesetRef, ViewObject,
};

#[test]
fn write_tmx_emits_tilesets_layers_and_objects() {
    let dir = TempDir::new("write_tmx");
    let map = TiledMap {
        width_tiles: 2,
        height_tiles: 1,
        tile_width: 20,
        tile_height: 20,
        background_color: Some("#112233".to_string()),
        tilesets: vec![TilesetRef {
            first_gid: 1,
            tsx_path: "tilesets/bg.tsx".to_string(),
        }],
        layers: vec![
            Layer::Tile(TileLayer {
                id: 1,
                name: "base".to_string(),
                width: 2,
                height: 1,
                data: vec![1, 2],
            }),
            Layer::Object(ObjectLayer {
                id: 2,
                name: "objects".to_string(),
                objects: vec![
                    MapObject::Instance(InstanceObject {
                        id: 10,
                        obj_type: "obj_test".to_string(),
                        x: 3.0,
                        y: 4.0,
                        width: 5.0,
                        height: 6.0,
                        instance_id: 77,
                        gid: Some(9),
                    }),
                    MapObject::TileObject(TileObjectData {
                        id: 11,
                        gid: 12,
                        x: 7.0,
                        y: 8.0,
                        width: 9.0,
                        height: 10.0,
                    }),
                    MapObject::View(ViewObject {
                        id: 12,
                        x: 1.0,
                        y: 2.0,
                        width: 30.0,
                        height: 40.0,
                        port_x: 5,
                        port_y: 6,
                        port_width: 70,
                        port_height: 80,
                    }),
                ],
            }),
        ],
        next_layer_id: 3,
        next_object_id: 13,
        speed: 60,
    };

    let path = dir.path().join("map.tmx");
    tmx::write_tmx(&map, &path).expect("write tmx");
    let xml = fs::read_to_string(path).expect("read tmx");

    assert!(xml.contains("backgroundcolor=\"#112233\""));
    assert!(xml.contains(r#"<tileset firstgid="1" source="tilesets/bg.tsx"/>"#));
    assert!(xml.contains(r#"<layer id="1" name="base" width="2" height="1">"#));
    assert!(xml.contains(">1,2<"));
    assert!(
        xml.contains(
            r#"<object id="10" gid="9" type="obj_test" x="3" y="4" width="5" height="6">"#
        )
    );
    assert!(xml.contains(r#"name="instanceId" type="int" value="77""#));
    assert!(xml.contains(r#"<object id="11" gid="12" x="7" y="8" width="9" height="10"/>"#));
    assert!(xml.contains(r#"<object id="12" class="view" x="1" y="2" width="30" height="40">"#));
}

#[test]
fn write_tsx_emits_expected_tileset_metadata() {
    let dir = TempDir::new("write_tsx");
    let tileset = Tileset {
        name: "bg".to_string(),
        tile_width: 20,
        tile_height: 20,
        image_path: "../textures/bg.png".to_string(),
        image_width: 40,
        image_height: 60,
        margin_x: 2,
        margin_y: 2,
        spacing_x: 4,
        spacing_y: 4,
        columns: 2,
        tile_count: 6,
        source_texture_page_index: 0,
        source_x: 0,
        source_y: 0,
    };

    let path = dir.path().join("bg.tsx");
    tsx::write_tsx(&tileset, &path).expect("write tsx");
    let xml = fs::read_to_string(path).expect("read tsx");

    assert!(xml.contains(r#"<tileset version="1.10" tiledversion="1.10.2" name="bg" tilewidth="20" tileheight="20" tilecount="6" columns="2" margin="2" spacing="4">"#));
    assert!(xml.contains(r#"<image source="../textures/bg.png" width="40" height="60"/>"#));
}
