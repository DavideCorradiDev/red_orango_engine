use super::AudioFormat;

pub trait Decoder {
    fn audio_format(&self) -> AudioFormat;
    fn byte_rate(&self) -> u32;
    fn sample_rate(&self) -> u32;
    fn byte_count(&self) -> usize;
    fn sample_count(&self) -> usize;
    fn byte_stream_position(&mut self) -> Result<u64, DecoderError>;
    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> Result<u64, DecoderError>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DecoderError>;

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, DecoderError> {
        buf.resize(self.byte_count() - self.byte_stream_position()? as usize, 0);
        self.read(&mut buf[..])
    }

    fn sample_stream_position(&mut self) -> Result<u64, DecoderError> {
        let byte_pos = self.byte_stream_position()?;
        let tbps = self.audio_format().total_bytes_per_sample() as u64;
        if byte_pos % tbps != 0 {
            return Err(DecoderError::CursorBetweenSamples);
        }
        Ok(byte_pos / tbps)
    }

    fn sample_seek(&mut self, pos: std::io::SeekFrom) -> Result<u64, DecoderError> {
        let tbps = self.audio_format().total_bytes_per_sample();
        let pos = match pos {
            std::io::SeekFrom::Start(v) => std::io::SeekFrom::Start(v * tbps as u64),
            std::io::SeekFrom::End(v) => std::io::SeekFrom::End(v * tbps as i64),
            std::io::SeekFrom::Current(v) => std::io::SeekFrom::Current(v * tbps as i64),
        };
        self.byte_seek(pos)
    }
}

#[derive(Debug)]
pub enum DecoderError {
    IoError(std::io::Error),
    CursorBetweenSamples,
}

impl std::fmt::Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecoderError::IoError(e) => write!(f, "Input / Output error ({})", e),
            DecoderError::CursorBetweenSamples => {
                write!(f, "The cursor position is in the middle of a sample")
            }
        }
    }
}

impl std::error::Error for DecoderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecoderError::IoError(e) => Some(e),
            DecoderError::CursorBetweenSamples => None,
        }
    }
}

impl From<std::io::Error> for DecoderError {
    fn from(e: std::io::Error) -> Self {
        DecoderError::IoError(e)
    }
}
