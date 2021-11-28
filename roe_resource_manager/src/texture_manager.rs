use std::{collections::HashMap, path::PathBuf};

use roe_graphics as gfx;

#[derive(Debug)]
pub struct TextureManager {
    assets_path: PathBuf,
    assets: HashMap<String, gfx::Texture>,
}

impl TextureManager {
    pub fn new(assets_path: PathBuf) -> Self {
        Self {
            assets_path,
            assets: HashMap::new(),
        }
    }

    pub fn get_asset_path(&self, asset_id: &str) -> PathBuf {
        let mut asset_path = self.assets_path.clone();
        asset_path.push(asset_id);
        asset_path
    }

    pub fn get(&self, asset_id: &str) -> Option<&gfx::Texture> {
        self.assets.get(asset_id)
    }

    pub fn insert(&mut self, instance: &gfx::Instance, texture_id: &str) -> Option<gfx::Texture> {
        // TODO: remove unwrap.
        let texture = gfx::Texture::from_image(
            &instance,
            &image::open(self.get_asset_path(texture_id))
                .unwrap()
                .into_rgba8(),
            gfx::TextureUsage::TEXTURE_BINDING,
        );
        self.assets.insert(texture_id.to_owned(), texture)
    }

    pub fn get_or_insert(
        &mut self,
        instance: &gfx::Instance,
        texture_id: &str,
    ) -> Result<&gfx::Texture, TextureManagerError> {
        if let None = self.get(texture_id) {
            self.insert(instance, texture_id);
        }
        Ok(self.get(texture_id).unwrap())
    }

    pub fn remove(&mut self, texture_id: &str) -> Option<gfx::Texture> {
        self.assets.remove(texture_id)
    }

    pub fn clear(&mut self) {
        self.assets.clear()
    }
}

#[derive(Debug)]
pub enum TextureManagerError {}
