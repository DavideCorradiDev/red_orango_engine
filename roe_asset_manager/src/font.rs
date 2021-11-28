use super::{AssetLoadError, AssetLoader, AssetManager};

use roe_text as txt;
use std::{collections::HashMap, path::Path, rc::Rc};

#[derive(Debug)]
pub struct Face {
    face: txt::Face,
    fonts: HashMap<String, txt::Font>,
}

impl Face {}

#[derive(Debug)]
pub struct FaceLoader {
    lib: Rc<txt::FontLibrary>,
}

impl FaceLoader {
    pub fn new(lib: Rc<txt::FontLibrary>) -> Self {
        Self { lib }
    }
}

impl AssetLoader<Face> for FaceLoader {
    fn load<P: AsRef<Path>>(&self, path: &P) -> Result<Face, AssetLoadError> {
        const FACE_INDEX: txt::FaceIndex = 0;
        let face = txt::Face::from_file(&self.lib, path, FACE_INDEX)?;
        Ok(Face {
            face,
            fonts: HashMap::new(),
        })
    }
}

impl From<txt::FontError> for AssetLoadError {
    fn from(e: txt::FontError) -> Self {
        Self::OtherError(format!("{}", e))
    }
}

pub type FaceManager = AssetManager<Face, FaceLoader>;

// TODO: tests.
