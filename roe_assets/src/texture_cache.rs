use roe_graphics as gfx;
use std::{collections::HashMap, path::PathBuf, rc::Rc};

#[derive(Debug)]
pub struct TextureCache {
    instance: Rc<gfx::Instance>,
    path: PathBuf,
    textures: HashMap<String, gfx::TextureView>,
}

impl TextureCache {
    pub fn new(instance: Rc<gfx::Instance>, path: PathBuf) -> Self {
        Self {
            instance,
            path,
            textures: HashMap::new(),
        }
    }

    fn get_path(&self, face_id: &str) -> PathBuf {
        let mut asset_path = self.path.clone();
        asset_path.push(face_id);
        asset_path
    }

    pub fn get(&self, file_id: &str) -> Option<&gfx::TextureView> {
        self.textures.get(file_id)
    }

    pub fn load(&mut self, file_id: &str) -> Result<Option<gfx::TextureView>, TextureCacheError> {
        let texture = gfx::Texture::from_image(
            &self.instance,
            &image::open(self.get_path(file_id))?.into_rgba8(),
            gfx::TextureUsage::TEXTURE_BINDING,
        );
        let texture_view = texture.create_view(&gfx::TextureViewDescriptor::default());
        Ok(self.textures.insert(String::from(file_id), texture_view))
    }

    pub fn get_or_load(&mut self, file_id: &str) -> Result<&gfx::TextureView, TextureCacheError> {
        if let None = self.get(file_id) {
            self.load(file_id)?;
        }
        Ok(self.get(file_id).unwrap())
    }

    pub fn remove(&mut self, file_id: &str) -> Option<gfx::TextureView> {
        self.textures.remove(file_id)
    }

    pub fn clear(&mut self) {
        self.textures.clear()
    }
}

#[derive(Debug)]
pub enum TextureCacheError {
    IoError(std::io::Error),
    ImageError(image::error::ImageError),
}

impl std::fmt::Display for TextureCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Input / Output error ({})", e),
            Self::ImageError(e) => write!(f, "Image error ({})", e),
        }
    }
}

impl std::error::Error for TextureCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::ImageError(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for TextureCacheError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<image::error::ImageError> for TextureCacheError {
    fn from(e: image::error::ImageError) -> Self {
        Self::ImageError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::*;

    fn create_texture_cache() -> TextureCache {
        let instance = Rc::new(gfx::Instance::new(&gfx::InstanceDescriptor::default()).unwrap());
        TextureCache::new(instance, PathBuf::from("data/pictures"))
    }

    #[test]
    #[serial_test::serial]
    fn creation() {
        let _ = create_texture_cache();
    }

    #[test]
    #[serial_test::serial]
    fn get_failure() {
        let texture_cache = create_texture_cache();
        expect_that!(&texture_cache.get("gioconda.jpg"), is_variant!(None));
        expect_that!(&texture_cache.get("triangles.png"), is_variant!(None));
    }

    #[test]
    #[serial_test::serial]
    fn get_success() {
        let mut texture_cache = create_texture_cache();
        texture_cache.load("gioconda.jpg").unwrap();
        expect_that!(&texture_cache.get("gioconda.jpg"), is_variant!(Some));
        expect_that!(&texture_cache.get("triangles.png"), is_variant!(None));
    }

    #[test]
    #[serial_test::serial]
    fn load() {
        let mut texture_cache = create_texture_cache();
        expect_that!(
            &texture_cache.load("gioconda.jpg").unwrap(),
            is_variant!(None)
        );
        expect_that!(
            &texture_cache.load("gioconda.jpg").unwrap(),
            is_variant!(Some)
        );
        expect_that!(
            &texture_cache.load("triangles.png").unwrap(),
            is_variant!(None)
        );
    }

    #[test]
    #[serial_test::serial]
    fn get_or_load() {
        let mut texture_cache = create_texture_cache();
        texture_cache.load("gioconda.jpg").unwrap();
        texture_cache.get_or_load("gioconda.jpg").unwrap();
        texture_cache.get_or_load("triangles.png").unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn remove() {
        let mut texture_cache = create_texture_cache();
        texture_cache.load("gioconda.jpg").unwrap();
        texture_cache.load("triangles.png").unwrap();
        expect_that!(&texture_cache.get("gioconda.jpg"), is_variant!(Some));
        expect_that!(&texture_cache.get("triangles.png"), is_variant!(Some));
        texture_cache.remove("gioconda.jpg");
        expect_that!(&texture_cache.get("gioconda.jpg"), is_variant!(None));
        expect_that!(&texture_cache.get("triangles.png"), is_variant!(Some));
    }

    #[test]
    #[serial_test::serial]
    fn clear() {
        let mut texture_cache = create_texture_cache();
        texture_cache.load("gioconda.jpg").unwrap();
        texture_cache.load("triangles.png").unwrap();
        expect_that!(&texture_cache.get("gioconda.jpg"), is_variant!(Some));
        expect_that!(&texture_cache.get("triangles.png"), is_variant!(Some));
        texture_cache.clear();
        expect_that!(&texture_cache.get("gioconda.jpg"), is_variant!(None));
        expect_that!(&texture_cache.get("triangles.png"), is_variant!(None));
    }
}
