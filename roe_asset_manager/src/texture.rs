use super::{AssetLoadError, AssetLoader, AssetManager};

use roe_graphics as gfx;
use std::{path::Path, rc::Rc};

pub use gfx::Texture;

#[derive(Debug)]
pub struct TextureLoader {
    instance: Rc<gfx::Instance>,
}

impl TextureLoader {
    pub fn new(instance: Rc<gfx::Instance>) -> Self {
        Self { instance }
    }
}

impl AssetLoader<Texture> for TextureLoader {
    fn load<P: AsRef<Path>>(&self, path: &P) -> Result<Texture, AssetLoadError> {
        Ok(gfx::Texture::from_image(
            &self.instance,
            &image::open(path)?.into_rgba8(),
            gfx::TextureUsage::TEXTURE_BINDING,
        ))
    }
}

impl From<image::error::ImageError> for AssetLoadError {
    fn from(e: image::error::ImageError) -> Self {
        Self::OtherError(format!("{}", e))
    }
}

pub type TextureManager = AssetManager<Texture, TextureLoader>;
