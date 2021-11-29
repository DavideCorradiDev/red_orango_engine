use roe_audio as audio;
use std::{collections::HashMap, path::{Path, PathBuf}, rc::Rc};

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

    pub fn load(&mut self, file_id: &str) -> Result<Option<audio::Buffer>, AudioBufferCacheError> {
        let path = self.get_path(file_id);
       let format = read_audio_format(&path);
       let input = std::io::BufReader::new(std::fs::File::open(&path)?);
       let audio_buffer = match format {
           AudioFormat::Wav => {
               audio::Buffer::from_decoder(&self.context, &mut audio::WavDecoder::new(input)?)?
           }
           AudioFormat::Ogg => {
               audio::Buffer::from_decoder(&self.context, &mut audio::OggDecoder::new(input)?)?
           }
           AudioFormat::Unknown => {
               return Err(AudioBufferCacheError::UnrecognizedAudioExtension)
           }
       };
        Ok(self.audio_buffers.insert(String::from(file_id), audio_buffer))
    }

    pub fn get_or_load(&mut self, file_id: &str) -> Result<&audio::Buffer, AudioBufferCacheError> {
        if let None = self.get(file_id) {
            self.load(file_id)?;
        }
        Ok(self.get(file_id).unwrap())
    }

    pub fn remove(&mut self, file_id: &str) -> Option<audio::Buffer> {
        self.audio_buffers.remove(&String::from(file_id))
    }

    pub fn clear(&mut self) {
        self.audio_buffers.clear()
    }
}

#[derive(Debug)]
pub enum AudioBufferCacheError {
    IoError(std::io::Error),
    AudioError(audio::Error),
    UnrecognizedAudioExtension,
}

impl std::fmt::Display for AudioBufferCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Input / Output error ({})", e),
            Self::AudioError(e) => write!(f, "Audio error ({})", e),
            Self::UnrecognizedAudioExtension => write!(f, "Unrecognized audio extension"),
        }
    }
}

impl std::error::Error for AudioBufferCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::AudioError(e) => Some(e),
            Self::UnrecognizedAudioExtension => None,
        }
    }
}

impl From<std::io::Error> for AudioBufferCacheError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<audio::Error> for AudioBufferCacheError {
    fn from(e: audio::Error) -> Self {
        Self::AudioError(e)
    }
}

impl From<audio::DecoderError> for AudioBufferCacheError {
    fn from(e: audio::DecoderError) -> Self {
        Self::from(audio::Error::from(e))
    }
}

// pub use audio::Buffer as AudioBuffer;
// 
// #[derive(Debug)]
// pub struct AudioBufferLoader {
//     context: Rc<audio::Context>,
// }
// 
// impl AudioBufferLoader {
//     pub fn new(context: Rc<audio::Context>) -> Self {
//         Self { context }
//     }
// }
// 
// impl AssetLoader<AudioBuffer> for AudioBufferLoader {
//     fn load<P: AsRef<Path>>(&self, path: &P) -> Result<AudioBuffer, AssetLoadError> {
//         let format = read_audio_format(path);
//         let input = std::io::BufReader::new(std::fs::File::open(path)?);
//         let audio_buffer = match format {
//             AudioFormat::Wav => {
//                 audio::Buffer::from_decoder(&self.context, &mut audio::WavDecoder::new(input)?)?
//             }
//             AudioFormat::Ogg => {
//                 audio::Buffer::from_decoder(&self.context, &mut audio::OggDecoder::new(input)?)?
//             }
//             AudioFormat::Unknown => {
//                 return Err(AssetLoadError::OtherError(String::from(
//                     "Unrecognized audio format",
//                 )));
//             }
//         };
//         Ok(audio_buffer)
//     }
// }
// 
// pub type AudioBufferManager = AssetManager<AudioBuffer, AudioBufferLoader>;
// 
// #[derive(Debug)]
// pub struct AudioStream {
//     stream_path: PathBuf,
// }
// 
// impl AudioStream {
//     pub fn create_decoder(&self) -> Result<Box<dyn audio::Decoder>, AssetLoadError> {
//         let format = read_audio_format(&self.stream_path);
//         let input = std::io::BufReader::new(std::fs::File::open(&self.stream_path)?);
//         let decoder = match format {
//             AudioFormat::Wav => Box::new(audio::WavDecoder::new(input)?) as Box<dyn audio::Decoder>,
//             AudioFormat::Ogg => Box::new(audio::OggDecoder::new(input)?) as Box<dyn audio::Decoder>,
//             AudioFormat::Unknown => {
//                 return Err(AssetLoadError::OtherError(String::from(
//                     "Unrecognized audio format",
//                 )));
//             }
//         };
//         Ok(decoder)
//     }
// }
// 
// #[derive(Debug)]
// pub struct AudioStreamLoader {}
// 
// impl AudioStreamLoader {
//     pub fn new() -> Self {
//         Self {}
//     }
// }
// 
// impl AssetLoader<AudioStream> for AudioStreamLoader {
//     fn load<P: AsRef<Path>>(&self, path: &P) -> Result<AudioStream, AssetLoadError> {
//         Ok(AudioStream {
//             stream_path: path.as_ref().to_path_buf(),
//         })
//     }
// }
// 
// pub type AudioStreamManager = AssetManager<AudioStream, AudioStreamLoader>;
// 
// impl From<audio::DecoderError> for AssetLoadError {
//     fn from(e: audio::DecoderError) -> Self {
//         Self::OtherError(format!("{}", e))
//     }
// }
// 
// impl From<audio::Error> for AssetLoadError {
//     fn from(e: audio::Error) -> Self {
//         Self::OtherError(format!("{}", e))
//     }
// }

// TODO: tests.