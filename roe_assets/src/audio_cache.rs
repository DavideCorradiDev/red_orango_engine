use roe_audio as audio;
use std::{
    borrow::BorrowMut,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

enum AudioFormat {
    Wav,
    Ogg,
    Unknown,
}

fn read_audio_format<P: AsRef<Path>>(path: P) -> AudioFormat {
    if let Some(extension) = path.as_ref().extension() {
        let extension = extension.to_ascii_lowercase();
        if extension == "wav" {
            return AudioFormat::Wav;
        }
        if extension == "ogg" {
            return AudioFormat::Ogg;
        }
    }
    AudioFormat::Unknown
}

fn load_decoder<P: AsRef<Path>>(path: P) -> Result<Box<dyn audio::Decoder>, audio::DecoderError> {
    let format = read_audio_format(&path);
    let input = std::io::BufReader::new(std::fs::File::open(&path)?);
    let decoder: Box<dyn audio::Decoder> = match format {
        AudioFormat::Wav => Box::new(audio::WavDecoder::new(input)?),
        AudioFormat::Ogg => Box::new(audio::OggDecoder::new(input)?),
        AudioFormat::Unknown => return Err(audio::DecoderError::Unimplemented),
    };
    Ok(decoder)
}

#[derive(Debug)]
pub struct AudioBufferCache {
    context: Rc<audio::Context>,
    path: PathBuf,
    audio_buffers: HashMap<String, audio::Buffer>,
}

impl AudioBufferCache {
    pub fn new(context: Rc<audio::Context>, path: PathBuf) -> Self {
        Self {
            context,
            path,
            audio_buffers: HashMap::new(),
        }
    }

    fn get_path(&self, face_id: &str) -> PathBuf {
        let mut asset_path = self.path.clone();
        asset_path.push(face_id);
        asset_path
    }

    pub fn get(&self, file_id: &str) -> Option<&audio::Buffer> {
        self.audio_buffers.get(file_id)
    }

    pub fn load(&mut self, file_id: &str) -> Result<Option<audio::Buffer>, AudioCacheError> {
        let mut decoder = load_decoder(self.get_path(file_id))?;
        let audio_buffer = audio::Buffer::from_decoder(
            &self.context,
            decoder.borrow_mut() as &mut dyn audio::Decoder,
        )?;
        Ok(self
            .audio_buffers
            .insert(String::from(file_id), audio_buffer))
    }

    pub fn get_or_load(&mut self, file_id: &str) -> Result<&audio::Buffer, AudioCacheError> {
        if let None = self.get(file_id) {
            self.load(file_id)?;
        }
        Ok(self.get(file_id).unwrap())
    }

    pub fn remove(&mut self, file_id: &str) -> Option<audio::Buffer> {
        self.audio_buffers.remove(file_id)
    }

    pub fn clear(&mut self) {
        self.audio_buffers.clear()
    }
}

pub struct AudioDecoderCache {
    path: PathBuf,
    audio_decoders: HashMap<String, Box<dyn audio::Decoder>>,
}

impl AudioDecoderCache {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            audio_decoders: HashMap::new(),
        }
    }

    fn get_path(&self, face_id: &str) -> PathBuf {
        let mut asset_path = self.path.clone();
        asset_path.push(face_id);
        asset_path
    }

    pub fn load(
        &mut self,
        file_id: &str,
    ) -> Result<Option<Box<dyn audio::Decoder>>, AudioCacheError> {
        let decoder = load_decoder(self.get_path(file_id))?;
        Ok(self.audio_decoders.insert(String::from(file_id), decoder))
    }

    pub fn insert(
        &mut self,
        file_id: &str,
        decoder: Box<dyn audio::Decoder>,
    ) -> Option<Box<dyn audio::Decoder>> {
        self.audio_decoders.insert(String::from(file_id), decoder)
    }

    pub fn remove(&mut self, file_id: &str) -> Option<Box<dyn audio::Decoder>> {
        self.audio_decoders.remove(file_id)
    }

    pub fn remove_or_load(
        &mut self,
        file_id: &str,
    ) -> Result<Box<dyn audio::Decoder>, AudioCacheError> {
        match self.remove(file_id) {
            Some(d) => Ok(d),
            None => Ok(load_decoder(self.get_path(file_id))?),
        }
    }

    pub fn clear(&mut self) {
        self.audio_decoders.clear()
    }
}

#[derive(Debug)]
pub enum AudioCacheError {
    IoError(std::io::Error),
    AudioError(audio::Error),
}

impl std::fmt::Display for AudioCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Input / Output error ({})", e),
            Self::AudioError(e) => write!(f, "Audio error ({})", e),
        }
    }
}

impl std::error::Error for AudioCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::AudioError(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for AudioCacheError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<audio::Error> for AudioCacheError {
    fn from(e: audio::Error) -> Self {
        Self::AudioError(e)
    }
}

impl From<audio::DecoderError> for AudioCacheError {
    fn from(e: audio::DecoderError) -> Self {
        Self::from(audio::Error::from(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::*;

    fn create_cache() -> AudioBufferCache {
        let device = audio::Device::default().unwrap();
        let context = Rc::new(audio::Context::default(&device).unwrap());
        AudioBufferCache::new(context, PathBuf::from("data/audio"))
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
        expect_that!(&cache.get("stereo-8-44100.wav"), is_variant!(None));
        expect_that!(&cache.get("stereo-16-44100.ogg"), is_variant!(None));
    }

    #[test]
    #[serial_test::serial]
    fn get_success() {
        let mut cache = create_cache();
        cache.load("stereo-8-44100.wav").unwrap();
        expect_that!(&cache.get("stereo-8-44100.wav"), is_variant!(Some));
        expect_that!(&cache.get("stereo-16-44100.ogg"), is_variant!(None));
    }

    #[test]
    #[serial_test::serial]
    fn load() {
        let mut cache = create_cache();
        expect_that!(
            &cache.load("stereo-8-44100.wav").unwrap(),
            is_variant!(None)
        );
        expect_that!(
            &cache.load("stereo-8-44100.wav").unwrap(),
            is_variant!(Some)
        );
        expect_that!(
            &cache.load("stereo-16-44100.ogg").unwrap(),
            is_variant!(None)
        );
    }

    #[test]
    #[serial_test::serial]
    fn get_or_load() {
        let mut cache = create_cache();
        cache.load("stereo-8-44100.wav").unwrap();
        cache.get_or_load("stereo-8-44100.wav").unwrap();
        cache.get_or_load("stereo-16-44100.ogg").unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn remove() {
        let mut cache = create_cache();
        cache.load("stereo-8-44100.wav").unwrap();
        cache.load("stereo-16-44100.ogg").unwrap();
        expect_that!(&cache.get("stereo-8-44100.wav"), is_variant!(Some));
        expect_that!(&cache.get("stereo-16-44100.ogg"), is_variant!(Some));
        expect_that!(&cache.remove("stereo-8-44100.wav"), is_variant!(Some));
        expect_that!(&cache.remove("stereo-8-44100.wav"), is_variant!(None));
        expect_that!(&cache.get("stereo-8-44100.wav"), is_variant!(None));
        expect_that!(&cache.get("stereo-16-44100.ogg"), is_variant!(Some));
    }

    #[test]
    #[serial_test::serial]
    fn clear() {
        let mut cache = create_cache();
        cache.load("stereo-8-44100.wav").unwrap();
        cache.load("stereo-16-44100.ogg").unwrap();
        expect_that!(&cache.get("stereo-8-44100.wav"), is_variant!(Some));
        expect_that!(&cache.get("stereo-16-44100.ogg"), is_variant!(Some));
        cache.clear();
        expect_that!(&cache.get("stereo-8-44100.wav"), is_variant!(None));
        expect_that!(&cache.get("stereo-16-44100.ogg"), is_variant!(None));
    }

    fn create_decoder_cache() -> AudioDecoderCache {
        AudioDecoderCache::new(PathBuf::from("data/audio"))
    }

    #[test]
    #[serial_test::serial]
    fn decoder_cache_creation() {
        let _ = create_decoder_cache();
    }

    #[test]
    #[serial_test::serial]
    fn decoder_cache_load() {
        let mut cache = create_decoder_cache();
        expect_that!(
            &cache.load("stereo-8-44100.wav").unwrap(),
            is_variant!(None)
        );
        expect_that!(
            &cache.load("stereo-8-44100.wav").unwrap(),
            is_variant!(Some)
        );
        expect_that!(
            &cache.load("stereo-16-44100.ogg").unwrap(),
            is_variant!(None)
        );
    }

    #[test]
    #[serial_test::serial]
    fn decoder_cache_remove_or_load() {
        let mut cache = create_decoder_cache();
        cache.load("stereo-8-44100.wav").unwrap();
        cache.remove_or_load("stereo-8-44100.wav").unwrap();
        cache.remove_or_load("stereo-16-44100.ogg").unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn decoder_cache_remove() {
        let mut cache = create_decoder_cache();
        cache.load("stereo-8-44100.wav").unwrap();
        cache.load("stereo-16-44100.ogg").unwrap();
        expect_that!(&cache.remove("stereo-8-44100.wav"), is_variant!(Some));
        expect_that!(&cache.remove("stereo-8-44100.wav"), is_variant!(None));
        expect_that!(&cache.remove("stereo-16-44100.ogg"), is_variant!(Some));
        expect_that!(&cache.remove("stereo-16-44100.ogg"), is_variant!(None));
    }

    #[test]
    #[serial_test::serial]
    fn decoder_cache_clear() {
        let mut cache = create_decoder_cache();
        cache.load("stereo-8-44100.wav").unwrap();
        cache.load("stereo-16-44100.ogg").unwrap();
        cache.clear();
        expect_that!(&cache.remove("stereo-8-44100.wav"), is_variant!(None));
        expect_that!(&cache.remove("stereo-16-44100.ogg"), is_variant!(None));
    }
}
