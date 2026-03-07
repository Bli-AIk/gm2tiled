use std::path::Path;

use anyhow::Context;
use image::DynamicImage;

/// Load a texture page PNG and crop a sub-image.
pub fn crop_background(
    texture_dir: &Path,
    texture_page_index: usize,
    src_x: u32,
    src_y: u32,
    src_width: u32,
    src_height: u32,
) -> anyhow::Result<DynamicImage> {
    let texture_path = texture_dir.join(format!("{texture_page_index}.png"));
    let img = image::open(&texture_path).with_context(|| {
        format!("Failed to open texture page {texture_page_index}: {texture_path:?}")
    })?;
    Ok(img.crop_imm(src_x, src_y, src_width, src_height))
}
