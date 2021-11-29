// use roe_graphics as gfx;
// use roe_text as txt;
// use std::{collections::HashMap, path::PathBuf, rc::Rc};
// 
// // #[derive(Debug)]
// // pub struct FaceLoader {
// //     lib: Rc<txt::FontLibrary>,
// // }
// //
// // impl FaceLoader {
// //     pub fn new(lib: Rc<txt::FontLibrary>) -> Self {
// //         Self { lib }
// //     }
// // }
// //
// // impl AssetLoader<Face> for FaceLoader {
// //     fn load<P: AsRef<Path>>(&self, path: &P) -> Result<Face, AssetLoadError> {
// //         const FACE_INDEX: txt::FaceIndex = 0;
// //         let face = txt::Face::from_file(&self.lib, path, FACE_INDEX)?;
// //         Ok(Face {
// //             face,
// //             fonts: HashMap::new(),
// //         })
// //     }
// // }
// //
// // impl From<txt::FontError> for AssetLoadError {
// //     fn from(e: txt::FontError) -> Self {
// //         Self::OtherError(format!("{}", e))
// //     }
// // }
// //
// // pub type FaceManager = AssetManager<Face, FaceLoader>;
// 
// #[derive(Debug, PartialEq, Eq, Clone, Hash)]
// struct FaceKey {
//     file_id: String,
//     face_index: txt::FaceIndex,
// }
// 
// impl FaceKey {
//     pub fn new(file_id: &str, face_index: txt::FaceIndex) -> Self {
//         Self {
//             file_id: String::from(file_id),
//             face_index,
//         }
//     }
// }
// 
// #[derive(Debug, PartialEq, Eq, Clone, Hash)]
// struct FontKey {
//     face_key: FaceKey,
//     font_size: txt::I26Dot6Size,
// }
// 
// impl FontKey {
//     pub fn new(file_id: &str, face_index: txt::FaceIndex, font_size: txt::FontSize) -> Self {
//         Self {
//             face_key: FaceKey::new(file_id, face_index),
//             font_size: txt::fsize_to_i26dot6(font_size),
//         }
//     }
// }
// 
// #[derive(Debug)]
// pub struct FontManager {
//     instance: Rc<gfx::Instance>,
//     lib: Rc<txt::FontLibrary>,
//     path: PathBuf,
//     character_set: Vec<char>,
//     faces: HashMap<FaceKey, txt::Face>,
//     fonts: HashMap<FontKey, txt::Font>,
// }
// 
// impl FontManager {
//     pub fn new(
//         instance: Rc<gfx::Instance>,
//         lib: Rc<txt::FontLibrary>,
//         path: PathBuf,
//         character_set: Vec<char>,
//     ) -> Self {
//         Self {
//             instance,
//             lib,
//             path,
//             character_set,
//             faces: HashMap::new(),
//             fonts: HashMap::new(),
//         }
//     }
// 
//     pub fn get_face_path(&self, face_id: &str) -> PathBuf {
//         let mut asset_path = self.path.clone();
//         asset_path.push(face_id);
//         asset_path
//     }
// 
//     pub fn get_face(&self, file_id: &str, face_index: txt::FaceIndex) -> Option<&txt::Face> {
//         self.faces.get(&FaceKey::new(file_id, face_index))
//     }
// 
//     pub fn load_face(
//         &mut self,
//         file_id: &str,
//         face_index: txt::FaceIndex,
//     ) -> Result<Option<txt::Face>, FontManagerError> {
//         let face = txt::Face::from_file(&self.lib, self.get_face_path(file_id), face_index)?;
//         Ok(self.faces.insert(FaceKey::new(file_id, face_index), face))
//     }
// 
//     pub fn get_or_load_face(
//         &mut self,
//         file_id: &str,
//         face_index: txt::FaceIndex,
//     ) -> Result<&txt::Face, FontManagerError> {
//         if let None = self.get_face(file_id, face_index) {
//             self.load_face(file_id, face_index)?;
//         }
//         Ok(self.get_face(file_id, face_index).unwrap())
//     }
// 
//     pub fn remove_face(&mut self, file_id: &str, face_index: txt::FaceIndex) -> Option<txt::Face> {
//         self.faces.remove(&FaceKey::new(file_id, face_index))
//     }
// 
//     pub fn clear_faces(&mut self) {
//         self.faces.clear()
//     }
// 
//     pub fn get_font(
//         &self,
//         file_id: &str,
//         face_index: txt::FaceIndex,
//         font_size: txt::FontSize,
//     ) -> Option<&txt::Font> {
//         self.fonts
//             .get(&FontKey::new(file_id, face_index, font_size))
//     }
// 
//     pub fn load_font(
//         &mut self,
//         file_id: &str,
//         face_index: txt::FaceIndex,
//         font_size: txt::FontSize,
//     ) -> Result<Option<txt::Font>, FontManagerError> {
//         let face = self.get_or_load_face(file_id, face_index)?;
//         let font = txt::Font::new(&self.instance, &face, font_size, &self.character_set)?;
//         Ok(self.fonts.insert(FontKey::new(file_id, face_index, font_size), font))
//     }
// 
//     // pub fn get_or_load(&mut self, file_id: &str) -> Result<&T, AssetLoadError> {
//     //     if let None = self.get(file_id) {
//     //         self.load(file_id)?;
//     //     }
//     //     Ok(self.get(file_id).unwrap())
//     // }
// 
//     // pub fn remove_face(&mut self, file_id: &str) -> Option<T> {
//     //     self.assets.remove(file_id)
//     // }
// 
//     // pub fn clear(&mut self) {
//     //     self.assets.clear()
//     // }
// }
// 
// #[derive(Debug)]
// pub enum FontManagerError {
//     IoError(std::io::Error),
//     FontError(txt::FontError),
// }
// 
// impl std::fmt::Display for FontManagerError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::IoError(e) => write!(f, "Input / Output error ({})", e),
//             Self::FontError(e) => write!(f, "Font error ({})", e),
//         }
//     }
// }
// 
// impl std::error::Error for FontManagerError {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         match self {
//             Self::IoError(e) => Some(e),
//             Self::FontError(_) => None,
//         }
//     }
// }
// 
// impl From<std::io::Error> for FontManagerError {
//     fn from(e: std::io::Error) -> Self {
//         Self::IoError(e)
//     }
// }
// 
// impl From<txt::FontError> for FontManagerError {
//     fn from(e: txt::FontError) -> Self {
//         Self::FontError(e)
//     }
// }
// 
// // TODO: tests.
// 