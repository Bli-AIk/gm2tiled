use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::Context;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};

use crate::model::Tileset;

pub fn write_tsx(tileset: &Tileset, output_path: &Path) -> anyhow::Result<()> {
    let file = File::create(output_path)
        .with_context(|| format!("Failed to create {output_path:?}"))?;
    let buf = BufWriter::new(file);
    let mut w = Writer::new_with_indent(buf, b' ', 2);

    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let tile_width = tileset.tile_width.to_string();
    let tile_height = tileset.tile_height.to_string();
    let tile_count = tileset.tile_count.to_string();
    let columns = tileset.columns.to_string();

    let mut ts = BytesStart::new("tileset");
    ts.push_attribute(("version", "1.10"));
    ts.push_attribute(("tiledversion", "1.10.2"));
    ts.push_attribute(("name", tileset.name.as_str()));
    ts.push_attribute(("tilewidth", tile_width.as_str()));
    ts.push_attribute(("tileheight", tile_height.as_str()));
    ts.push_attribute(("tilecount", tile_count.as_str()));
    ts.push_attribute(("columns", columns.as_str()));
    w.write_event(Event::Start(ts))?;

    let img_width = tileset.image_width.to_string();
    let img_height = tileset.image_height.to_string();

    let mut img = BytesStart::new("image");
    img.push_attribute(("source", tileset.image_path.as_str()));
    img.push_attribute(("width", img_width.as_str()));
    img.push_attribute(("height", img_height.as_str()));
    w.write_event(Event::Empty(img))?;

    w.write_event(Event::End(BytesEnd::new("tileset")))?;
    Ok(())
}
