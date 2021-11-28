use super::{ResourceLoader, Error};

use roe_graphics as gfx;

pub struct TextureLoader {}

impl ResourceLoader<gfx::Texture> for TextureLoader {
    type Resource = gfx::Texture;

    fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self::Resource, Error>
    {

    }
}