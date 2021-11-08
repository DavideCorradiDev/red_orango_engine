use super::AudioFormat;

pub trait Decoder {
    fn audio_format(&self) -> AudioFormat;

    fn byte_rate(&self) -> u32 {
        self.sample_rate() * self.audio_format().total_bytes_per_sample()
    }

    fn byte_count(&self) -> usize {
        self.sample_count() * self.audio_format().total_bytes_per_sample() as usize
    }

    fn byte_stream_position(&mut self) -> Result<u64, DecoderError> {
        Ok(self.sample_stream_position()? * self.audio_format().total_bytes_per_sample() as u64)
    }

    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> Result<u64, DecoderError>;

    fn sample_rate(&self) -> u32;
    fn sample_count(&self) -> usize;

    fn sample_stream_position(&mut self) -> Result<u64, DecoderError>;

    fn sample_seek(&mut self, pos: std::io::SeekFrom) -> Result<u64, DecoderError> {
        let tbps = self.audio_format().total_bytes_per_sample();
        let pos = match pos {
            std::io::SeekFrom::Start(v) => std::io::SeekFrom::Start(v * tbps as u64),
            std::io::SeekFrom::End(v) => std::io::SeekFrom::End(v * tbps as i64),
            std::io::SeekFrom::Current(v) => std::io::SeekFrom::Current(v * tbps as i64),
        };
        let byte_count = self.byte_seek(pos)?;
        Ok(byte_count / tbps as u64)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DecoderError>;

    fn read_to_end(&mut self) -> Result<Vec<u8>, DecoderError> {
        let tbps = self.audio_format().total_bytes_per_sample() as usize;
        let size = self.byte_count() - self.byte_stream_position()? as usize;
        assert!(size % tbps == 0);
        let mut buf = vec![0; size];
        self.read(&mut buf[..])?;
        Ok(buf)
    }

    fn read_all(&mut self) -> Result<Vec<u8>, DecoderError> {
        if self.byte_stream_position()? != 0 {
            self.byte_seek(std::io::SeekFrom::Start(0))?;
        }
        self.read_to_end()
    }
}

#[derive(Debug)]
pub enum DecoderError {
    IoError(std::io::Error),
    InvalidEncoding(String),
    InvalidHeader(String),
    InvalidData(String),
    Unimplemented,
}

impl std::fmt::Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Input / Output error ({})", e),
            Self::InvalidEncoding(e) => write!(f, "Invalid encoding ({})", e),
            Self::InvalidHeader(e) => write!(f, "Invalid header ({})", e),
            Self::InvalidData(e) => write!(f, "Invalid data ({})", e),
            Self::Unimplemented => write!(f, "Unimplemented"),
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
