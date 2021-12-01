use roe_graphics as gfx;
use std::{collections::HashMap, path::PathBuf, rc::Rc};

#[derive(Debug)]
pub struct TextureCache {
    instance: Rc<gfx::Instance>,
    path: PathBuf,
    textures: HashMap<String, Rc<gfx::TextureView>>,
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

    pub fn get(&self, file_id: &str) -> Option<Rc<gfx::TextureView>> {
        match self.textures.get(file_id) {
            Some(t) => Some(Rc::clone(t)),
            None => None
        }
    }

    pub fn load(&mut self, file_id: &str) -> Result<Option<Rc<gfx::TextureView>>, TextureCacheError> {
        let texture = gfx::Texture::from_image(
            &self.instance,
            &image::open(self.get_path(file_id))?.into_rgba8(),
            gfx::TextureUsage::TEXTURE_BINDING,
        );
        let texture_view = texture.create_view(&gfx::TextureViewDescriptor::default());
        Ok(self.textures.insert(String::from(file_id), Rc::new(texture_view)))
    }

    pub fn get_or_load(&mut self, file_id: &str) -> Result<Rc<gfx::TextureView>, TextureCacheError> {
        if let None = self.get(file_id) {
            self.load(file_id)?;
        }
        Ok(self.get(file_id).unwrap())
    }

    pub fn remove(&mut self, file_id: &str) -> Option<Rc<gfx::TextureView>> {
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

// TODO: tests.
