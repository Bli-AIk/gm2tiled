use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::Context;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};

use crate::model::{Layer, MapObject, ObjectLayer, TileLayer, TiledMap};

pub fn write_tmx(map: &TiledMap, output_path: &Path) -> anyhow::Result<()> {
    let file =
        File::create(output_path).with_context(|| format!("Failed to create {output_path:?}"))?;
    let buf = BufWriter::new(file);
    let mut w = Writer::new_with_indent(buf, b' ', 2);

    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    write_map_start(&mut w, map)?;
    write_properties(&mut w, map.speed)?;

    for ts in &map.tilesets {
        let first_gid = ts.first_gid.to_string();
        let mut elem = BytesStart::new("tileset");
        elem.push_attribute(("firstgid", first_gid.as_str()));
        elem.push_attribute(("source", ts.tsx_path.as_str()));
        w.write_event(Event::Empty(elem))?;
    }

    for layer in &map.layers {
        match layer {
            Layer::Tile(tl) => write_tile_layer(&mut w, tl)?,
            Layer::Object(ol) => write_object_layer(&mut w, ol)?,
        }
    }

    w.write_event(Event::End(BytesEnd::new("map")))?;
    Ok(())
}

fn write_map_start<W: std::io::Write>(w: &mut Writer<W>, map: &TiledMap) -> anyhow::Result<()> {
    let width = map.width_tiles.to_string();
    let height = map.height_tiles.to_string();
    let tilewidth = map.tile_width.to_string();
    let tileheight = map.tile_height.to_string();
    let nlid = map.next_layer_id.to_string();
    let noid = map.next_object_id.to_string();

    let mut elem = BytesStart::new("map");
    elem.push_attribute(("version", "1.10"));
    elem.push_attribute(("tiledversion", "1.10.2"));
    elem.push_attribute(("orientation", "orthogonal"));
    elem.push_attribute(("renderorder", "right-down"));
    elem.push_attribute(("width", width.as_str()));
    elem.push_attribute(("height", height.as_str()));
    elem.push_attribute(("tilewidth", tilewidth.as_str()));
    elem.push_attribute(("tileheight", tileheight.as_str()));
    elem.push_attribute(("infinite", "0"));
    elem.push_attribute(("nextlayerid", nlid.as_str()));
    elem.push_attribute(("nextobjectid", noid.as_str()));
    if let Some(ref bg) = map.background_color {
        elem.push_attribute(("backgroundcolor", bg.as_str()));
    }
    w.write_event(Event::Start(elem))?;
    Ok(())
}

fn write_properties<W: std::io::Write>(w: &mut Writer<W>, speed: u32) -> anyhow::Result<()> {
    let speed_str = speed.to_string();
    w.write_event(Event::Start(BytesStart::new("properties")))?;
    let mut prop = BytesStart::new("property");
    prop.push_attribute(("name", "speed"));
    prop.push_attribute(("type", "int"));
    prop.push_attribute(("value", speed_str.as_str()));
    w.write_event(Event::Empty(prop))?;
    w.write_event(Event::End(BytesEnd::new("properties")))?;
    Ok(())
}

fn write_tile_layer<W: std::io::Write>(w: &mut Writer<W>, layer: &TileLayer) -> anyhow::Result<()> {
    let id = layer.id.to_string();
    let width = layer.width.to_string();
    let height = layer.height.to_string();

    let mut elem = BytesStart::new("layer");
    elem.push_attribute(("id", id.as_str()));
    elem.push_attribute(("name", layer.name.as_str()));
    elem.push_attribute(("width", width.as_str()));
    elem.push_attribute(("height", height.as_str()));
    w.write_event(Event::Start(elem))?;

    let mut data_elem = BytesStart::new("data");
    data_elem.push_attribute(("encoding", "csv"));
    w.write_event(Event::Start(data_elem))?;

    let csv = layer
        .data
        .iter()
        .map(|gid| gid.to_string())
        .collect::<Vec<_>>()
        .join(",");
    w.write_event(Event::Text(BytesText::new(&csv)))?;
    w.write_event(Event::End(BytesEnd::new("data")))?;
    w.write_event(Event::End(BytesEnd::new("layer")))?;
    Ok(())
}

fn write_object_layer<W: std::io::Write>(
    w: &mut Writer<W>,
    layer: &ObjectLayer,
) -> anyhow::Result<()> {
    let id = layer.id.to_string();
    let mut elem = BytesStart::new("objectgroup");
    elem.push_attribute(("id", id.as_str()));
    elem.push_attribute(("name", layer.name.as_str()));
    w.write_event(Event::Start(elem))?;

    for obj in &layer.objects {
        match obj {
            MapObject::Instance(inst) => write_instance_object(w, inst)?,
            MapObject::TileObject(tile_obj) => write_tile_object(w, tile_obj)?,
            MapObject::View(view) => write_view_object(w, view)?,
        }
    }

    w.write_event(Event::End(BytesEnd::new("objectgroup")))?;
    Ok(())
}

fn write_instance_object<W: std::io::Write>(
    w: &mut Writer<W>,
    inst: &crate::model::InstanceObject,
) -> anyhow::Result<()> {
    let id = inst.id.to_string();
    let x = inst.x.to_string();
    let y = inst.y.to_string();
    let width = inst.width.to_string();
    let height = inst.height.to_string();
    let iid = inst.instance_id.to_string();
    let gid_str = inst.gid.map(|g| g.to_string());

    let mut elem = BytesStart::new("object");
    elem.push_attribute(("id", id.as_str()));
    if let Some(ref g) = gid_str {
        elem.push_attribute(("gid", g.as_str()));
    }
    elem.push_attribute(("type", inst.obj_type.as_str()));
    elem.push_attribute(("x", x.as_str()));
    elem.push_attribute(("y", y.as_str()));
    elem.push_attribute(("width", width.as_str()));
    elem.push_attribute(("height", height.as_str()));
    w.write_event(Event::Start(elem))?;

    w.write_event(Event::Start(BytesStart::new("properties")))?;
    let mut prop = BytesStart::new("property");
    prop.push_attribute(("name", "instanceId"));
    prop.push_attribute(("type", "int"));
    prop.push_attribute(("value", iid.as_str()));
    w.write_event(Event::Empty(prop))?;
    w.write_event(Event::End(BytesEnd::new("properties")))?;
    w.write_event(Event::End(BytesEnd::new("object")))?;
    Ok(())
}

fn write_tile_object<W: std::io::Write>(
    w: &mut Writer<W>,
    tile_obj: &crate::model::TileObjectData,
) -> anyhow::Result<()> {
    let id = tile_obj.id.to_string();
    let gid = tile_obj.gid.to_string();
    let x = tile_obj.x.to_string();
    let y = tile_obj.y.to_string();
    let width = tile_obj.width.to_string();
    let height = tile_obj.height.to_string();

    let mut elem = BytesStart::new("object");
    elem.push_attribute(("id", id.as_str()));
    elem.push_attribute(("gid", gid.as_str()));
    elem.push_attribute(("x", x.as_str()));
    elem.push_attribute(("y", y.as_str()));
    elem.push_attribute(("width", width.as_str()));
    elem.push_attribute(("height", height.as_str()));
    w.write_event(Event::Empty(elem))?;
    Ok(())
}

fn write_view_object<W: std::io::Write>(
    w: &mut Writer<W>,
    view: &crate::model::ViewObject,
) -> anyhow::Result<()> {
    let id = view.id.to_string();
    let x = view.x.to_string();
    let y = view.y.to_string();
    let width = view.width.to_string();
    let height = view.height.to_string();

    let mut elem = BytesStart::new("object");
    elem.push_attribute(("id", id.as_str()));
    elem.push_attribute(("class", "view"));
    elem.push_attribute(("x", x.as_str()));
    elem.push_attribute(("y", y.as_str()));
    elem.push_attribute(("width", width.as_str()));
    elem.push_attribute(("height", height.as_str()));
    w.write_event(Event::Start(elem))?;

    write_view_properties(w, view)?;

    w.write_event(Event::End(BytesEnd::new("object")))?;
    Ok(())
}

fn write_view_properties<W: std::io::Write>(
    w: &mut Writer<W>,
    view: &crate::model::ViewObject,
) -> anyhow::Result<()> {
    let port_x = view.port_x.to_string();
    let port_y = view.port_y.to_string();
    let port_w = view.port_width.to_string();
    let port_h = view.port_height.to_string();

    w.write_event(Event::Start(BytesStart::new("properties")))?;
    write_int_property(w, "portX", &port_x)?;
    write_int_property(w, "portY", &port_y)?;
    write_int_property(w, "portWidth", &port_w)?;
    write_int_property(w, "portHeight", &port_h)?;
    w.write_event(Event::End(BytesEnd::new("properties")))?;
    Ok(())
}

fn write_int_property<W: std::io::Write>(
    w: &mut Writer<W>,
    name: &str,
    value: &str,
) -> anyhow::Result<()> {
    let mut prop = BytesStart::new("property");
    prop.push_attribute(("name", name));
    prop.push_attribute(("type", "int"));
    prop.push_attribute(("value", value));
    w.write_event(Event::Empty(prop))?;
    Ok(())
}
