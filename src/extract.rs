use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Context;

use crate::schema::{BackgroundDef, RoomData, SpriteDef};

/// Write output dir path to `/tmp/gm2tiled_outdir`, then run utmt.
pub fn run_utmt(data_win: &Path, extract_dir: &Path, scripts_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(extract_dir)?;
    fs::write(
        "/tmp/gm2tiled_outdir",
        extract_dir.to_string_lossy().as_bytes(),
    )
    .context("Failed to write /tmp/gm2tiled_outdir")?;

    let script_path = scripts_dir.join("extract_data.csx");
    let output = std::process::Command::new("utmt")
        .arg("load")
        .arg(data_win)
        .arg("--scripts")
        .arg(&script_path)
        .output()
        .context("Failed to run utmt. Make sure it is installed and in PATH.")?;

    if !output.status.success() {
        eprintln!("utmt stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("utmt stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("utmt exited with non-zero status: {}", output.status);
    }

    Ok(())
}

/// Load backgrounds from extracted JSON into a name→def map.
pub fn load_backgrounds(extract_dir: &Path) -> anyhow::Result<HashMap<String, BackgroundDef>> {
    let json_path = extract_dir.join("backgrounds.json");
    let content =
        fs::read_to_string(&json_path).with_context(|| format!("Failed to read {json_path:?}"))?;
    let list: Vec<BackgroundDef> =
        serde_json::from_str(&content).context("Failed to parse backgrounds.json")?;
    Ok(list.into_iter().map(|b| (b.name.clone(), b)).collect())
}

/// Load full sprite catalog from extracted JSON.
pub fn load_sprites(extract_dir: &Path) -> anyhow::Result<Vec<SpriteDef>> {
    let json_path = extract_dir.join("sprites.json");
    let content =
        fs::read_to_string(&json_path).with_context(|| format!("Failed to read {json_path:?}"))?;
    serde_json::from_str(&content).context("Failed to parse sprites.json")
}

/// Load a single room from extracted JSON.
pub fn load_room(extract_dir: &Path, room_name: &str) -> anyhow::Result<RoomData> {
    let json_path = extract_dir.join("rooms").join(format!("{room_name}.json"));
    let content = fs::read_to_string(&json_path)
        .with_context(|| format!("Failed to read room '{room_name}': {json_path:?}"))?;
    serde_json::from_str(&content).with_context(|| format!("Failed to parse room '{room_name}'"))
}

/// List available room names from extracted output.
pub fn list_rooms(extract_dir: &Path) -> anyhow::Result<Vec<String>> {
    let rooms_dir = extract_dir.join("rooms");
    let mut rooms = Vec::new();
    for entry in fs::read_dir(&rooms_dir)
        .with_context(|| format!("Failed to read rooms directory {rooms_dir:?}"))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            rooms.push(stem.to_string());
        }
    }
    rooms.sort();
    Ok(rooms)
}
