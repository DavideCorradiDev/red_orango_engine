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

    pub fn insert(
        &mut self,
        instance: &gfx::Instance,
        texture_id: &str,
    ) -> Result<Option<gfx::Texture>, TextureManagerError> {
        let texture = gfx::Texture::from_image(
            &instance,
            &image::open(self.get_asset_path(texture_id))?.into_rgba8(),
            gfx::TextureUsage::TEXTURE_BINDING,
        );
        Ok(self.assets.insert(texture_id.to_owned(), texture))
    }

    pub fn get_or_insert(
        &mut self,
        instance: &gfx::Instance,
        texture_id: &str,
    ) -> Result<&gfx::Texture, TextureManagerError> {
        if let None = self.get(texture_id) {
            self.insert(instance, texture_id)?;
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
pub enum TextureManagerError {
    IoError(std::io::Error),
    ImageError(image::error::ImageError),
}

impl std::fmt::Display for TextureManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Input / Output error ({})", e),
            Self::ImageError(e) => write!(f, "Image error ({})", e),
        }
    }
}

impl std::error::Error for TextureManagerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TextureManagerError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<image::error::ImageError> for TextureManagerError {
    fn from(e: image::error::ImageError) -> Self {
        Self::ImageError(e)
    }
}

// TODO unit tests.
