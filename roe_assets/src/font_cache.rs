use roe_graphics as gfx;
use roe_text as txt;
use std::{collections::HashMap, path::PathBuf, rc::Rc};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct FontKey {
    file_id: String,
    face_index: txt::FaceIndex,
    font_size: txt::I26Dot6Size,
}

impl FontKey {
    pub fn new(file_id: &str, face_index: txt::FaceIndex, font_size: txt::FontSize) -> Self {
        Self {
            file_id: String::from(file_id),
            face_index,
            font_size: txt::fsize_to_i26dot6(font_size),
        }
    }
}

#[derive(Debug)]
pub struct FontCache {
    instance: Rc<gfx::Instance>,
    lib: Rc<txt::FontLibrary>,
    path: PathBuf,
    character_set: Vec<char>,
    fonts: HashMap<FontKey, txt::Font>,
}

impl FontCache {
    pub fn new(
        instance: Rc<gfx::Instance>,
        lib: Rc<txt::FontLibrary>,
        path: PathBuf,
        character_set: Vec<char>,
    ) -> Self {
        Self {
            instance,
            lib,
            path,
            character_set,
            fonts: HashMap::new(),
        }
    }

    fn get_face_path(&self, face_id: &str) -> PathBuf {
        let mut asset_path = self.path.clone();
        asset_path.push(face_id);
        asset_path
    }

    fn load_face(
        &mut self,
        file_id: &str,
        face_index: txt::FaceIndex,
    ) -> Result<txt::Face, FontCacheError> {
        Ok(txt::Face::from_file(
            &self.lib,
            self.get_face_path(file_id),
            face_index,
        )?)
    }

    pub fn get(
        &self,
        file_id: &str,
        face_index: txt::FaceIndex,
        font_size: txt::FontSize,
    ) -> Option<&txt::Font> {
        self.fonts
            .get(&FontKey::new(file_id, face_index, font_size))
    }

    pub fn load(
        &mut self,
        file_id: &str,
        face_index: txt::FaceIndex,
        font_size: txt::FontSize,
    ) -> Result<Option<txt::Font>, FontCacheError> {
        let face = self.load_face(file_id, face_index)?;
        let font = txt::Font::new(&self.instance, &face, font_size, &self.character_set)?;
        Ok(self
            .fonts
            .insert(FontKey::new(file_id, face_index, font_size), font))
    }

    pub fn get_or_load(
        &mut self,
        file_id: &str,
        face_index: txt::FaceIndex,
        font_size: txt::FontSize,
    ) -> Result<&txt::Font, FontCacheError> {
        if let None = self.get(file_id, face_index, font_size) {
            self.load(file_id, face_index, font_size)?;
        }
        Ok(self.get(file_id, face_index, font_size).unwrap())
    }

    pub fn remove(
        &mut self,
        file_id: &str,
        face_index: txt::FaceIndex,
        font_size: txt::FontSize,
    ) -> Option<txt::Font> {
        self.fonts
            .remove(&FontKey::new(file_id, face_index, font_size))
    }

    pub fn clear(&mut self) {
        self.fonts.clear()
    }
}

#[derive(Debug)]
pub enum FontCacheError {
    IoError(std::io::Error),
    FontError(txt::FontError),
}

impl std::fmt::Display for FontCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Input / Output error ({})", e),
            Self::FontError(e) => write!(f, "Font error ({})", e),
        }
    }
}

impl std::error::Error for FontCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::FontError(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for FontCacheError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<txt::FontError> for FontCacheError {
    fn from(e: txt::FontError) -> Self {
        Self::FontError(e)
    }
}

// TODO: tests.
#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::*;

    fn create_cache() -> FontCache {
        let instance = Rc::new(gfx::Instance::new(&gfx::InstanceDescriptor::default()).unwrap());
        let lib = Rc::new(txt::FontLibrary::new().unwrap());
        FontCache::new(
            instance,
            lib,
            PathBuf::from("data/fonts"),
            txt::character_set::english(),
        )
    }

    #[test]
    #[serial_test::serial]
    fn creation() {
        let _ = create_cache();
    }

    #[test]
    #[serial_test::serial]
    fn get_failure() {
        let cache = create_cache();
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 10.), is_variant!(None));
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 15.), is_variant!(None));
    }

    #[test]
    #[serial_test::serial]
    fn get_success() {
        let mut cache = create_cache();
        cache.load("Roboto-Regular.ttf", 0, 10.).unwrap();
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 10.), is_variant!(Some));
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 15.), is_variant!(None));
    }

    #[test]
    #[serial_test::serial]
    fn load() {
        let mut cache = create_cache();
        expect_that!(
            &cache.load("Roboto-Regular.ttf", 0, 10.).unwrap(),
            is_variant!(None)
        );
        expect_that!(
            &cache.load("Roboto-Regular.ttf", 0, 10.).unwrap(),
            is_variant!(Some)
        );
        expect_that!(
            &cache.load("Roboto-Regular.ttf", 0, 15.).unwrap(),
            is_variant!(None)
        );
    }

    #[test]
    #[serial_test::serial]
    fn get_or_load() {
        let mut cache = create_cache();
        cache.load("Roboto-Regular.ttf", 0, 10.).unwrap();
        cache.get_or_load("Roboto-Regular.ttf", 0, 10.).unwrap();
        cache.get_or_load("Roboto-Regular.ttf", 0, 15.).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn remove() {
        let mut cache = create_cache();
        cache.load("Roboto-Regular.ttf", 0, 10.).unwrap();
        cache.load("Roboto-Regular.ttf", 0, 15.).unwrap();
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 10.), is_variant!(Some));
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 15.), is_variant!(Some));
        expect_that!(
            &cache.remove("Roboto-Regular.ttf", 0, 10.),
            is_variant!(Some)
        );
        expect_that!(
            &cache.remove("Roboto-Regular.ttf", 0, 10.),
            is_variant!(None)
        );
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 10.), is_variant!(None));
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 15.), is_variant!(Some));
    }

    #[test]
    #[serial_test::serial]
    fn clear() {
        let mut cache = create_cache();
        cache.load("Roboto-Regular.ttf", 0, 10.).unwrap();
        cache.load("Roboto-Regular.ttf", 0, 15.).unwrap();
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 10.), is_variant!(Some));
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 15.), is_variant!(Some));
        cache.clear();
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 10.), is_variant!(None));
        expect_that!(&cache.get("Roboto-Regular.ttf", 0, 15.), is_variant!(None));
    }
}
