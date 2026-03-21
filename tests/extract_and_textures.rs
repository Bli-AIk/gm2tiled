mod common;

use std::fs;

use common::{TempDir, rgba, write_png};
use gm2tiled::{extract, textures::TexturePageCache};
use image::RgbaImage;

#[test]
fn extract_loaders_parse_catalogs_and_sort_rooms() {
    let dir = TempDir::new("extract_loaders");
    fs::create_dir_all(dir.path().join("rooms")).expect("create rooms");

    fs::write(
        dir.path().join("backgrounds.json"),
        r#"[{"name":"bg","texturePageIndex":0,"sourceX":1,"sourceY":2,"sourceWidth":32,"sourceHeight":48,"gms2TileWidth":16,"gms2TileHeight":16}]"#,
    )
    .expect("write backgrounds");
    fs::write(
        dir.path().join("sprites.json"),
        r#"[{"name":"spr","frames":[{"texturePageIndex":1,"sourceX":3,"sourceY":4,"sourceWidth":5,"sourceHeight":6}]}]"#,
    )
    .expect("write sprites");
    fs::write(
        dir.path().join("rooms").join("b_room.json"),
        r#"{"width":40,"height":30,"speed":60,"backgroundColor":4278190080,"drawBackgroundColor":true}"#,
    )
    .expect("write room b");
    fs::write(
        dir.path().join("rooms").join("a_room.json"),
        r#"{"width":10,"height":20,"speed":15,"backgroundColor":4294901760,"drawBackgroundColor":false}"#,
    )
    .expect("write room a");
    fs::write(dir.path().join("rooms").join("ignore.txt"), "nope").expect("write junk");

    let backgrounds = extract::load_backgrounds(dir.path()).expect("load backgrounds");
    assert_eq!(backgrounds["bg"].gms2_tile_width, 16);
    let sprites = extract::load_sprites(dir.path()).expect("load sprites");
    assert_eq!(sprites[0].frames[0].source_height, 6);
    let room = extract::load_room(dir.path(), "a_room").expect("load room");
    assert_eq!(room.width, 10);
    let rooms = extract::list_rooms(dir.path()).expect("list rooms");
    assert_eq!(rooms, vec!["a_room".to_string(), "b_room".to_string()]);
}

#[test]
fn texture_page_cache_reuses_loaded_pages() {
    let dir = TempDir::new("texture_cache");
    let mut image = RgbaImage::new(2, 2);
    image.put_pixel(0, 0, rgba(255, 0, 0, 255));
    image.put_pixel(1, 0, rgba(0, 255, 0, 255));
    image.put_pixel(0, 1, rgba(0, 0, 255, 255));
    image.put_pixel(1, 1, rgba(255, 255, 0, 255));
    write_png(&dir.path().join("0.png"), &image);

    let mut cache = TexturePageCache::new(dir.path());
    let crop = cache.crop(0, 1, 0, 1, 1).expect("first crop").to_rgba8();
    assert_eq!(crop.get_pixel(0, 0), &rgba(0, 255, 0, 255));

    fs::remove_file(dir.path().join("0.png")).expect("remove cached source");
    let second = cache.crop(0, 0, 1, 1, 1).expect("second crop").to_rgba8();
    assert_eq!(second.get_pixel(0, 0), &rgba(0, 0, 255, 255));
}

#[test]
fn texture_page_cache_reports_missing_pages() {
    let dir = TempDir::new("texture_cache_missing");
    let mut cache = TexturePageCache::new(dir.path());
    let error = cache
        .crop(99, 0, 0, 1, 1)
        .expect_err("missing page should fail");
    assert!(error.to_string().contains("Failed to open texture page 99"));
}
