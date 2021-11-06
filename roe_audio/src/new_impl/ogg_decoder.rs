use ogg::reading::PacketReader;

use super::{AudioFormat, Decoder};

pub struct OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    packet_reader: PacketReader<T>,
    ident_header: lewton::header::IdentHeader,
    comment_header: lewton::header::CommentHeader,
    setup_header: lewton::header::SetupHeader,
    stream_serial: u32,
    format: AudioFormat,
    sample_count: usize,
}

impl<T> OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    pub fn new(input: T) -> Result<Self, DecoderError> {
        let mut packet_reader = PacketReader::new(input);
        let ((ident_header, comment_header, setup_header), stream_serial) =
            lewton::inside_ogg::read_headers(&mut packet_reader)?;
        const BYTES_PER_SAMPLE: u32 = 2;
        let format = AudioFormat::new(ident_header.audio_channels as u32, BYTES_PER_SAMPLE);
        let sample_count = Self::compute_byte_count(&mut packet_reader)? / format.bytes_per_sample() as usize;
        Ok(Self {
            packet_reader,
            ident_header,
            comment_header,
            setup_header,
            stream_serial,
            format,
            sample_count,
        })
    }

    fn compute_byte_count(packet_reader: &mut PacketReader<T>) -> Result<usize, DecoderError> {
        let mut maybe_packet = packet_reader.read_packet()?;
        let mut byte_count = 0;
        while maybe_packet.is_some() {
            let packet = maybe_packet.unwrap();
            byte_count += packet.data.len();
            maybe_packet = packet_reader.read_packet()?;
        }
        Ok(byte_count)
    }
}

impl<T> Decoder for OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    fn audio_format(&self) -> AudioFormat {
        self.format
    }

    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        Ok(0)
    }

    fn sample_rate(&self) -> u32 {
        self.ident_header.audio_sample_rate
    }

    fn sample_count(&self) -> usize {
        self.sample_count
    }

    fn sample_stream_position(&mut self) -> std::io::Result<u64> {
        Ok(0)
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(0)
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

impl From<lewton::VorbisError> for DecoderError {
    fn from(e: lewton::VorbisError) -> Self {
        match e {
            lewton::VorbisError::BadAudio(e) => DecoderError::from(e),
            lewton::VorbisError::BadHeader(e) => DecoderError::from(e),
            lewton::VorbisError::OggError(e) => DecoderError::from(e),
        }
    }
}

impl From<lewton::audio::AudioReadError> for DecoderError {
    fn from(e: lewton::audio::AudioReadError) -> Self {
        DecoderError::InvalidData(format!("{}", e))
    }
}

impl From<lewton::header::HeaderReadError> for DecoderError {
    fn from(e: lewton::header::HeaderReadError) -> Self {
        match e {
            lewton::header::HeaderReadError::NotVorbisHeader => {
                DecoderError::InvalidEncoding(format!("{}", e))
            }
            _ => DecoderError::InvalidHeader(format!("{}", e)),
        }
    }
}

impl From<ogg::reading::OggReadError> for DecoderError {
    fn from(e: ogg::reading::OggReadError) -> Self {
        match e {
            ogg::reading::OggReadError::ReadError(e) => DecoderError::IoError(e),
            ogg::reading::OggReadError::NoCapturePatternFound => {
                DecoderError::InvalidEncoding(format!("{}", e))
            }
            _ => DecoderError::InvalidData(format!("{}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    // TODO: use DecoderError for wav decoder.
    // TODO: match expectations in wav test.
    #[test]
    fn invalid_input_file() {
        let file = std::fs::File::open("data/audio/not-an-audio-file.txt").unwrap();
        let buf = std::io::BufReader::new(file);
        let result = OggDecoder::new(buf);
        expect_that!(&result, is_variant!(Result::Err));
        if let Err(e) = result {
            expect_that!(&e, is_variant!(DecoderError::InvalidEncoding));
        }
    }

    #[test]
    fn mono16_loading() {
        let file = std::fs::File::open("data/audio/mono-16-44100.ogg").unwrap();
        let buf = std::io::BufReader::new(file);
        let decoder = OggDecoder::new(buf).unwrap();
        expect_that!(&decoder.audio_format(), eq(AudioFormat::Mono16));
        expect_that!(&decoder.byte_count(), eq(1420 * 2));
        expect_that!(&decoder.sample_count(), eq(1420));
        expect_that!(&decoder.byte_rate(), eq(44100 * 2));
        expect_that!(&decoder.sample_rate(), eq(44100));
    }
}
