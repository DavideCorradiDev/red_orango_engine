use super::{AssetLoadError, AssetLoader, AssetManager};

use roe_audio as audio;
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

enum AudioFormat {
    Wav,
    Ogg,
    Unknown,
}

fn read_audio_format<P: AsRef<Path>>(path: &P) -> AudioFormat {
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
    fn load<P: AsRef<Path>>(&self, path: &P) -> Result<AudioBuffer, AssetLoadError> {
        let format = read_audio_format(path);
        let input = std::io::BufReader::new(std::fs::File::open(path)?);
        let audio_buffer = match format {
            AudioFormat::Wav => {
                audio::Buffer::from_decoder(&self.context, &mut audio::WavDecoder::new(input)?)?
            }
            AudioFormat::Ogg => {
                audio::Buffer::from_decoder(&self.context, &mut audio::OggDecoder::new(input)?)?
            }
            AudioFormat::Unknown => {
                return Err(AssetLoadError::OtherError(String::from(
                    "Unrecognized audio format",
                )));
            }
        };
        Ok(audio_buffer)
    }
}

pub type AudioBufferManager = AssetManager<AudioBuffer, AudioBufferLoader>;

#[derive(Debug)]
pub struct AudioStream {
    stream_path: PathBuf,
}

impl AudioStream {
    pub fn create_decoder(&self) -> Result<Box<dyn audio::Decoder>, AssetLoadError> {
        let format = read_audio_format(&self.stream_path);
        let input = std::io::BufReader::new(std::fs::File::open(&self.stream_path)?);
        let decoder = match format {
            AudioFormat::Wav => Box::new(audio::WavDecoder::new(input)?) as Box<dyn audio::Decoder>,
            AudioFormat::Ogg => Box::new(audio::OggDecoder::new(input)?) as Box<dyn audio::Decoder>,
            AudioFormat::Unknown => {
                return Err(AssetLoadError::OtherError(String::from(
                    "Unrecognized audio format",
                )));
            }
        };
        Ok(decoder)
    }
}

#[derive(Debug)]
pub struct AudioStreamLoader {}

impl AudioStreamLoader {
    pub fn new() -> Self {
        Self {}
    }
}

impl AssetLoader<AudioStream> for AudioStreamLoader {
    fn load<P: AsRef<Path>>(&self, path: &P) -> Result<AudioStream, AssetLoadError> {
        Ok(AudioStream {
            stream_path: path.as_ref().to_path_buf(),
        })
    }
}

pub type AudioStreamManager = AssetManager<AudioStream, AudioStreamLoader>;

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

// TODO: tests.