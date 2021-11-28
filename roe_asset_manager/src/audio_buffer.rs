use super::{AssetLoadError, AssetLoader, AssetManager};

use roe_audio as audio;
use std::{path::Path, rc::Rc};

pub use audio::Buffer as AudioBuffer;

#[derive(Debug)]
pub struct AudioBufferLoader {
    context: Rc<audio::Context>,
}

impl AudioBufferLoader {
    pub fn new(context: Rc<audio::Context>) -> Self {
        Self { context }
    }
}

impl AssetLoader<AudioBuffer> for AudioBufferLoader {
    fn load<P: AsRef<Path>>(&self, path: P) -> Result<AudioBuffer, AssetLoadError> {
        let audio_buffer = audio::Buffer::from_decoder(
            &self.context,
            &mut audio::WavDecoder::new(std::io::BufReader::new(std::fs::File::open(path)?))?,
        )?;
        Ok(audio_buffer)
    }
}

impl From<audio::DecoderError> for AssetLoadError {
    fn from(e: audio::DecoderError) -> Self {
        Self::OtherError(format!("{}", e))
    }
}

impl From<audio::Error> for AssetLoadError {
    fn from(e: audio::Error) -> Self {
        Self::OtherError(format!("{}", e))
    }
}

pub type AudioBufferManager = AssetManager<AudioBuffer, AudioBufferLoader>;
