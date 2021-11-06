use super::{AudioFormat, Decoder};

pub struct OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    decoder: vorbis::Decoder<T>,
    format: AudioFormat,
    sample_rate: u32,
    sample_count: usize,
}

impl<T> OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    // TODO: return result.
    pub fn new(input: T) -> Result<Self, DecoderError> {
        let mut decoder = vorbis::Decoder::new(input)?;
        let mut channel_count = 0;
        let mut sample_rate = 0;
        let mut sample_count = 0;
        for packet in decoder.packets() {
            let packet = packet?;
            channel_count = packet.channels as u32;
            sample_rate = packet.rate as u32;
            sample_count += packet.data.len();
        }
        assert!(channel_count == 1 || channel_count == 2);
        assert!(sample_count % channel_count as usize == 0);
        sample_count /= channel_count as usize;

        const BYTES_PER_SAMPLE: u32 = 2;
        let format = AudioFormat::new(channel_count, BYTES_PER_SAMPLE);

        Ok(Self {
            decoder,
            format,
            sample_rate,
            sample_count
        })
    }
}

#[derive(Debug)]
pub enum DecoderError {
    IoError(std::io::Error),
    InvalidEncoding(String),
    InvalidHeader(String),
    InvalidData(String),
    Unimplemented
}

impl std::fmt::Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Input / Output error ({})", e),
            Self::InvalidEncoding(e) => write!(f, "Invalid encoding ({})", e),
            Self::InvalidHeader(e) => write!(f, "Invalid header ({})", e),
            Self::InvalidData(e) => write!(f, "Invalid data ({})", e),
            Self::Unimplemented => write!(f, "Unimplemented")
        }
    }
}

impl std::error::Error for DecoderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DecoderError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<vorbis::VorbisError> for DecoderError {
    fn from(e: vorbis::VorbisError) -> Self {
        match e {
            vorbis::VorbisError::ReadError(e) => DecoderError::IoError(e),
            vorbis::VorbisError::NotVorbis => DecoderError::InvalidEncoding(String::from("Input data isn't vorbis")),
            vorbis::VorbisError::VersionMismatch => DecoderError::InvalidEncoding(String::from("Vorbis version mismatch ({})")),
            vorbis::VorbisError::BadHeader => DecoderError::InvalidHeader(String::from("Bad vorbis header")),
            vorbis::VorbisError::Hole => DecoderError::InvalidData(String::from("Vorbis data hole")),
            vorbis::VorbisError::InvalidSetup => DecoderError::InvalidData(String::from("Invalid vorbis setup")),
            vorbis::VorbisError::Unimplemented => DecoderError::Unimplemented,
        }
    }
}