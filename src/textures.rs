use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use image::DynamicImage;

pub struct TexturePageCache {
    texture_dir: PathBuf,
    pages: HashMap<usize, DynamicImage>,
}

impl TexturePageCache {
    pub fn new(texture_dir: &Path) -> Self {
        Self {
            texture_dir: texture_dir.to_path_buf(),
            pages: HashMap::new(),
        }
    }

    /// Load a texture page PNG once, then crop sub-images from the cached page.
    pub fn crop(
        &mut self,
        texture_page_index: usize,
        src_x: u32,
        src_y: u32,
        src_width: u32,
        src_height: u32,
    ) -> anyhow::Result<DynamicImage> {
        if !self.pages.contains_key(&texture_page_index) {
            let texture_path = self.texture_dir.join(format!("{texture_page_index}.png"));
            let img = image::open(&texture_path).with_context(|| {
                format!("Failed to open texture page {texture_page_index}: {texture_path:?}")
            })?;
            self.pages.insert(texture_page_index, img);
        }

        Ok(self.pages[&texture_page_index].crop_imm(src_x, src_y, src_width, src_height))
    }
}
