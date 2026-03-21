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
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        eprintln!("utmt stdout:\n{stdout}");
        eprintln!("utmt stderr:\n{stderr}");
        anyhow::bail!("utmt exited with non-zero status: {}", output.status);
    }
    if utmt_output_indicates_failure(&stdout, &stderr) {
        eprintln!("utmt stdout:\n{stdout}");
        eprintln!("utmt stderr:\n{stderr}");
        anyhow::bail!("utmt reported a script or runtime failure despite exiting successfully");
    }
    validate_extract_output(extract_dir).with_context(|| {
        format!(
            "utmt did not produce the expected extraction files in {:?}",
            extract_dir
        )
    })?;

    Ok(())
}

fn utmt_output_indicates_failure(stdout: &str, stderr: &str) -> bool {
    [stdout, stderr].iter().any(|stream| {
        stream.contains("CompilationErrorException")
            || stream.contains("Unhandled exception:")
            || stream.contains("error CS")
    })
}

fn validate_extract_output(extract_dir: &Path) -> anyhow::Result<()> {
    let required_files = [
        extract_dir.join("backgrounds.json"),
        extract_dir.join("sprites.json"),
    ];
    for path in required_files {
        if !path.is_file() {
            anyhow::bail!("Missing required extracted file {:?}", path);
        }
    }

    let rooms_dir = extract_dir.join("rooms");
    if !rooms_dir.is_dir() {
        anyhow::bail!("Missing extracted rooms directory {:?}", rooms_dir);
    }
    let has_room_json = fs::read_dir(&rooms_dir)
        .with_context(|| format!("Failed to read extracted rooms directory {:?}", rooms_dir))?
        .filter_map(Result::ok)
        .any(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"));
    if !has_room_json {
        anyhow::bail!("No extracted room JSON files found in {:?}", rooms_dir);
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

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{utmt_output_indicates_failure, validate_extract_output};

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let path = env::temp_dir().join(format!("gm2tiled_{prefix}_{unique}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn remove_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn utmt_failure_marker_detection_catches_script_errors() {
        assert!(utmt_output_indicates_failure(
            "Microsoft.CodeAnalysis.Scripting.CompilationErrorException: boom",
            ""
        ));
        assert!(utmt_output_indicates_failure(
            "",
            "Unhandled exception: bad"
        ));
        assert!(utmt_output_indicates_failure(
            "",
            "error CS0656: Missing compiler required member"
        ));
        assert!(!utmt_output_indicates_failure(
            "gm2tiled extraction complete.",
            ""
        ));
    }

    #[test]
    fn validate_extract_output_requires_expected_files() {
        let dir = temp_dir("extract_validation");
        let error = validate_extract_output(&dir).expect_err("missing files should fail");
        assert!(
            error
                .to_string()
                .contains("Missing required extracted file")
        );

        fs::write(dir.join("backgrounds.json"), "[]").expect("backgrounds");
        fs::write(dir.join("sprites.json"), "[]").expect("sprites");
        fs::create_dir_all(dir.join("rooms")).expect("rooms dir");
        let error = validate_extract_output(&dir).expect_err("missing room json should fail");
        assert!(
            error
                .to_string()
                .contains("No extracted room JSON files found")
        );

        fs::write(dir.join("rooms").join("room_a.json"), "{}").expect("room json");
        validate_extract_output(&dir).expect("valid extract output");
        remove_temp_dir(&dir);
    }
}
