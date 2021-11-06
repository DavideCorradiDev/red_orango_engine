use super::{AudioFormat, Decoder};

use bytemuck::Zeroable;

#[repr(C, packed)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct WavSignature {
    id: [u8; 4],
    size: u32,
    form: [u8; 4],
}

unsafe impl bytemuck::Zeroable for WavSignature {
    fn zeroed() -> Self {
        Self {
            id: [0; 4],
            size: 0,
            form: [0; 4],
        }
    }
}

unsafe impl bytemuck::Pod for WavSignature {}

#[repr(C, packed)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct WavChunkSignature {
    id: [u8; 4],
    size: u32,
}

unsafe impl bytemuck::Zeroable for WavChunkSignature {
    fn zeroed() -> Self {
        Self {
            id: [0; 4],
            size: 0,
        }
    }
}

unsafe impl bytemuck::Pod for WavChunkSignature {}

#[repr(C, packed)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct WavFormatChunk {
    signature: WavChunkSignature,
    format: u16,
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

unsafe impl bytemuck::Zeroable for WavFormatChunk {
    fn zeroed() -> Self {
        Self {
            signature: WavChunkSignature::zeroed(),
            format: 0,
            channels: 0,
            sample_rate: 0,
            byte_rate: 0,
            block_align: 0,
            bits_per_sample: 0,
        }
    }
}

unsafe impl bytemuck::Pod for WavFormatChunk {}

#[derive(Debug)]
pub struct WavDecoder<T: std::io::Read + std::io::Seek> {
    input: T,
    format: AudioFormat,
    sample_rate: u32,
    sample_count: usize,
    byte_data_offset: u64,
}

impl<T> WavDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    pub fn new(mut input: T) -> Result<Self, WavDecoderCreationError> {
        input.seek(std::io::SeekFrom::Start(0)).unwrap();

        let mut signature = WavSignature::zeroed();
        input.read_exact(bytemuck::bytes_of_mut(&mut signature))?;
        {
            let id_str = std::str::from_utf8(&signature.id).unwrap();
            if id_str != "RIFF" {
                return Err(WavDecoderCreationError::InvalidId(String::from(id_str)));
            }
            let form_str = std::str::from_utf8(&signature.form).unwrap();
            if form_str != "WAVE" {
                return Err(WavDecoderCreationError::InvalidForm(String::from(form_str)));
            }
        }

        let mut format_chunk = WavFormatChunk::zeroed();
        input.read_exact(bytemuck::bytes_of_mut(&mut format_chunk))?;
        {
            let id_str = std::str::from_utf8(&format_chunk.signature.id).unwrap();
            if id_str != "fmt " {
                return Err(WavDecoderCreationError::InvalidFormatChunkId(String::from(
                    id_str,
                )));
            }
            if format_chunk.channels != 1 && format_chunk.channels != 2 {
                return Err(WavDecoderCreationError::InvalidChannelCount(
                    format_chunk.channels,
                ));
            }
            if format_chunk.bits_per_sample != 8 && format_chunk.bits_per_sample != 16 {
                return Err(WavDecoderCreationError::InvalidBitsPerSample(
                    format_chunk.bits_per_sample,
                ));
            }
            if format_chunk.byte_rate
                != format_chunk.sample_rate
                    * format_chunk.channels as u32
                    * (format_chunk.bits_per_sample / 8) as u32
            {
                return Err(WavDecoderCreationError::InvalidByteRate(
                    format_chunk.byte_rate,
                ));
            }
            if format_chunk.block_align
                != format_chunk.channels * (format_chunk.bits_per_sample / 8)
            {
                return Err(WavDecoderCreationError::InvalidBlockAlignment(
                    format_chunk.block_align,
                ));
            }
        }

        let byte_count = loop {
            let mut chunk_signature = WavChunkSignature::zeroed();
            input.read_exact(bytemuck::bytes_of_mut(&mut chunk_signature))?;
            let chunk_id = std::str::from_utf8(&chunk_signature.id).unwrap();
            if chunk_id == "data" {
                break chunk_signature.size;
            }
            input.seek(std::io::SeekFrom::Current(chunk_signature.size as i64))?;
        };
        let bytes_per_sample = format_chunk.bits_per_sample / 8;
        let format = AudioFormat::new(format_chunk.channels as u32, bytes_per_sample as u32);
        let sample_rate = format_chunk.sample_rate;
        let sample_count = (byte_count / format.total_bytes_per_sample()) as usize;
        let byte_data_offset = input.stream_position()?;
        Ok(Self {
            input,
            format,
            sample_rate,
            sample_count,
            byte_data_offset,
        })
    }
}

impl<T> Decoder for WavDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    fn audio_format(&self) -> AudioFormat {
        self.format
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn sample_count(&self) -> usize {
        self.sample_count
    }

    fn sample_stream_position(&mut self) -> std::io::Result<u64> {
        let input_pos = self.input.stream_position()?;
        let tbps = self.audio_format().total_bytes_per_sample() as u64;
        assert!(input_pos >= self.byte_data_offset);
        let input_pos = input_pos - self.byte_data_offset;
        assert!(input_pos % tbps == 0);
        Ok(input_pos / tbps)
    }

    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let byte_count = self.byte_count() as i64;
        let target_pos = match pos {
            std::io::SeekFrom::Start(v) => v as i64,
            std::io::SeekFrom::End(v) => byte_count + v,
            std::io::SeekFrom::Current(v) => self.byte_stream_position()? as i64 + v,
        };
        let target_pos = std::cmp::max(0, std::cmp::min(target_pos, byte_count)) as u64;

        let tbps = self.audio_format().total_bytes_per_sample() as u64;
        assert!(
            target_pos % tbps == 0,
            "Invalid seek offset ({})",
            target_pos
        );

        let count = self
            .input
            .seek(std::io::SeekFrom::Start(self.byte_data_offset + target_pos))?;
        Ok(count - self.byte_data_offset)
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let tbps = self.audio_format().total_bytes_per_sample() as usize;
        assert!(
            buf.len() % tbps == 0,
            "Invalid buffer length ({})",
            buf.len()
        );
        let count = self.input.read(buf)?;
        Ok(count)
    }
}

#[derive(Debug)]
pub enum WavDecoderCreationError {
    IoError(std::io::Error),
    InvalidId(String),
    InvalidForm(String),
    InvalidFormatChunkId(String),
    InvalidChannelCount(u16),
    InvalidBitsPerSample(u16),
    InvalidByteRate(u32),
    InvalidBlockAlignment(u16),
}

impl std::fmt::Display for WavDecoderCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Input / Output error ({})", e),
            Self::InvalidId(e) => write!(f, "Invalid id ({})", e),
            Self::InvalidForm(e) => write!(f, "Invalid form ({})", e),
            Self::InvalidFormatChunkId(e) => write!(f, "Invalid format chunk id ({})", e),
            Self::InvalidChannelCount(e) => write!(f, "Invalid channel count ({})", e),
            Self::InvalidBitsPerSample(e) => write!(f, "Invalid bits per sample ({})", e),
            Self::InvalidByteRate(e) => write!(f, "Invalid byte rate ({})", e),
            Self::InvalidBlockAlignment(e) => write!(f, "Invalid block alignment ({})", e),
        }
    }
}

impl std::error::Error for WavDecoderCreationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for WavDecoderCreationError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    fn invalid_input_file() {
        let file = std::fs::File::open("data/audio/not-an-audio-file.txt").unwrap();
        let buf = std::io::BufReader::new(file);
        expect_that!(&WavDecoder::new(buf), is_variant!(Result::Err));
    }

    #[test]
    fn mono8_loading() {
        let file = std::fs::File::open("data/audio/mono-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let decoder = WavDecoder::new(buf).unwrap();
        expect_that!(&decoder.audio_format(), eq(AudioFormat::Mono8));
        expect_that!(&decoder.byte_count(), eq(21231));
        expect_that!(&decoder.sample_count(), eq(21231));
        expect_that!(&decoder.byte_rate(), eq(44100));
        expect_that!(&decoder.sample_rate(), eq(44100));
    }

    #[test]
    fn mono8_byte_seek() {
        let file = std::fs::File::open("data/audio/mono-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(13)).unwrap(),
            eq(13)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(13));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(13));

        // From current positive.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(5)).unwrap(),
            eq(18)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(18));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(18));

        // From current negative.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-7)).unwrap(),
            eq(11)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(11));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(11));

        // From end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(-10)).unwrap(),
            eq(21221)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(21221));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21221));

        // Beyond end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(42)).unwrap(),
            eq(21231)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(21231));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21231));

        // Before start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    fn mono8_sample_seek() {
        let file = std::fs::File::open("data/audio/mono-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(13)).unwrap(),
            eq(13)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(13));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(13));

        // From current positive.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(5)).unwrap(),
            eq(18)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(18));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(18));

        // From current negative.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-7)).unwrap(),
            eq(11)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(11));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(11));

        // From end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(-10)).unwrap(),
            eq(21221)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(21221));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21221));

        // Beyond end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(42)).unwrap(),
            eq(21231)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(21231));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21231));

        // Before start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    fn mono8_read() {
        let file = std::fs::File::open("data/audio/mono-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        let mut buf = vec![0; 7];

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(7));
        expect_that!(&buf, eq(vec![178, 178, 178, 178, 177, 177, 177]));

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(7));
        expect_that!(&buf, eq(vec![177, 177, 177, 176, 176, 128, 80]));

        decoder.byte_seek(std::io::SeekFrom::End(-3)).unwrap();
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(21228));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21228));

        // Unable to read the whole buffer because at the end: the remaining elements
        // aren't overwritten!
        expect_that!(&decoder.read(&mut buf).unwrap(), eq(3));
        expect_that!(&buf, eq(vec![128, 128, 128, 176, 176, 128, 80]));
    }

    #[test]
    fn mono8_read_to_end() {
        let file = std::fs::File::open("data/audio/mono-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(573)).unwrap();
        let content = decoder.read_to_end().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count() - 573));
    }

    #[test]
    fn mono8_read_all() {
        let file = std::fs::File::open("data/audio/mono-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(573)).unwrap();
        let content = decoder.read_all().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count()));
    }

    #[test]
    fn mono16_loading() {
        let file = std::fs::File::open("data/audio/mono-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let decoder = WavDecoder::new(buf).unwrap();
        expect_that!(&decoder.audio_format(), eq(AudioFormat::Mono16));
        expect_that!(&decoder.byte_count(), eq(21231 * 2));
        expect_that!(&decoder.sample_count(), eq(21231));
        expect_that!(&decoder.byte_rate(), eq(44100 * 2));
        expect_that!(&decoder.sample_rate(), eq(44100));
    }

    #[test]
    fn mono16_byte_seek() {
        let file = std::fs::File::open("data/audio/mono-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(12)).unwrap(),
            eq(12)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(12));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(6));

        // From current positive.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(4)).unwrap(),
            eq(16)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(16));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(8));

        // From current negative.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-8)).unwrap(),
            eq(8)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(8));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(4));

        // From end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(-12)).unwrap(),
            eq(42450)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42450));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21225));

        // Beyond end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(40)).unwrap(),
            eq(42462)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42462));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21231));

        // Before start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    #[should_panic(expected = "Invalid seek offset (3)")]
    fn mono16_byte_seek_invalid_offset() {
        let file = std::fs::File::open("data/audio/mono-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(3)).unwrap();
    }

    #[test]
    fn mono16_sample_seek() {
        let file = std::fs::File::open("data/audio/mono-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(3)).unwrap(),
            eq(3)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(6));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(3));

        // From current positive.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(1)).unwrap(),
            eq(4)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(8));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(4));

        // From current negative.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-2)).unwrap(),
            eq(2)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(4));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(2));

        // From end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(-3)).unwrap(),
            eq(21228)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42456));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21228));

        // Beyond end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(10)).unwrap(),
            eq(21231)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42462));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21231));

        // Before start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    fn mono16_read() {
        let file = std::fs::File::open("data/audio/mono-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        let mut buf = vec![0; 8];

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![87, 49, 44, 49, 1, 49, 214, 48]));

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![171, 48, 129, 48, 86, 48, 43, 48]));

        decoder.byte_seek(std::io::SeekFrom::End(-4)).unwrap();
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42458));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21229));

        // Unable to read the whole buffer because at the end: the remaining elements
        // aren't overwritten!
        expect_that!(&decoder.read(&mut buf).unwrap(), eq(4));
        expect_that!(&buf, eq(vec![0, 0, 0, 0, 86, 48, 43, 48]));
    }

    #[test]
    #[should_panic(expected = "Invalid buffer length (7)")]
    fn mono16_read_invalid_buffer_length() {
        let file = std::fs::File::open("data/audio/mono-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        let mut buf = vec![0; 7];
        decoder.read(&mut buf).unwrap();
    }

    #[test]
    fn mono16_read_to_end() {
        let file = std::fs::File::open("data/audio/mono-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
        let content = decoder.read_to_end().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count() - 572));
    }

    #[test]
    fn mono16_read_all() {
        let file = std::fs::File::open("data/audio/mono-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
        let content = decoder.read_all().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count()));
    }

    #[test]
    fn stereo8_loading() {
        let file = std::fs::File::open("data/audio/stereo-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let decoder = WavDecoder::new(buf).unwrap();
        expect_that!(&decoder.audio_format(), eq(AudioFormat::Stereo8));
        expect_that!(&decoder.byte_count(), eq(21231 * 2));
        expect_that!(&decoder.sample_count(), eq(21231));
        expect_that!(&decoder.byte_rate(), eq(44100 * 2));
        expect_that!(&decoder.sample_rate(), eq(44100));
    }

    #[test]
    fn stereo8_byte_seek() {
        let file = std::fs::File::open("data/audio/stereo-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(12)).unwrap(),
            eq(12)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(12));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(6));

        // From current positive.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(4)).unwrap(),
            eq(16)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(16));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(8));

        // From current negative.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-8)).unwrap(),
            eq(8)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(8));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(4));

        // From end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(-12)).unwrap(),
            eq(42450)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42450));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21225));

        // Beyond end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(40)).unwrap(),
            eq(42462)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42462));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21231));

        // Before start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    #[should_panic(expected = "Invalid seek offset (3)")]
    fn stereo8_byte_seek_invalid_offset() {
        let file = std::fs::File::open("data/audio/stereo-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(3)).unwrap();
    }

    #[test]
    fn stereo8_sample_seek() {
        let file = std::fs::File::open("data/audio/stereo-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(3)).unwrap(),
            eq(3)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(6));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(3));

        // From current positive.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(1)).unwrap(),
            eq(4)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(8));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(4));

        // From current negative.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-2)).unwrap(),
            eq(2)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(4));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(2));

        // From end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(-3)).unwrap(),
            eq(21228)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42456));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21228));

        // Beyond end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(10)).unwrap(),
            eq(21231)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42462));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21231));

        // Before start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    fn stereo8_read() {
        let file = std::fs::File::open("data/audio/stereo-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        let mut buf = vec![0; 8];

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![163, 163, 163, 163, 163, 163, 163, 163]));

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![162, 162, 162, 162, 162, 162, 162, 162]));

        decoder.byte_seek(std::io::SeekFrom::End(-4)).unwrap();
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(42458));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21229));

        // Unable to read the whole buffer because at the end: the remaining elements
        // aren't overwritten!
        expect_that!(&decoder.read(&mut buf).unwrap(), eq(4));
        expect_that!(&buf, eq(vec![128, 128, 128, 128, 162, 162, 162, 162]));
    }

    #[test]
    #[should_panic(expected = "Invalid buffer length (7)")]
    fn stereo8_read_invalid_buffer_length() {
        let file = std::fs::File::open("data/audio/stereo-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        let mut buf = vec![0; 7];
        decoder.read(&mut buf).unwrap();
    }

    #[test]
    fn stereo8_read_to_end() {
        let file = std::fs::File::open("data/audio/stereo-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
        let content = decoder.read_to_end().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count() - 572));
    }

    #[test]
    fn stereo8_read_all() {
        let file = std::fs::File::open("data/audio/stereo-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
        let content = decoder.read_all().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count()));
    }

    #[test]
    fn stereo16_loading() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let decoder = WavDecoder::new(buf).unwrap();
        expect_that!(&decoder.audio_format(), eq(AudioFormat::Stereo16));
        expect_that!(&decoder.byte_count(), eq(21231 * 4));
        expect_that!(&decoder.sample_count(), eq(21231));
        expect_that!(&decoder.byte_rate(), eq(44100 * 4));
        expect_that!(&decoder.sample_rate(), eq(44100));
    }

    #[test]
    fn stereo16_byte_seek() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(12)).unwrap(),
            eq(12)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(12));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(3));

        // From current positive.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(4)).unwrap(),
            eq(16)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(16));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(4));

        // From current negative.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-8)).unwrap(),
            eq(8)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(8));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(2));

        // From end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(-12)).unwrap(),
            eq(84912)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(84912));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21228));

        // Beyond end.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::End(40)).unwrap(),
            eq(84924)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(84924));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21231));

        // Before start.
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.byte_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    #[should_panic(expected = "Invalid seek offset (3)")]
    fn stereo16_byte_seek_invalid_offset() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(3)).unwrap();
    }

    #[test]
    fn stereo16_sample_seek() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();

        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));

        // From start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(3)).unwrap(),
            eq(3)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(12));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(3));

        // From current positive.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(1)).unwrap(),
            eq(4)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(16));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(4));

        // From current negative.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-2)).unwrap(),
            eq(2)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(8));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(2));

        // From end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(-3)).unwrap(),
            eq(21228)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(84912));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21228));

        // Beyond end.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::End(10)).unwrap(),
            eq(21231)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(84924));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21231));

        // Before start.
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Start(0)).unwrap(),
            eq(0)
        );
        expect_that!(
            &decoder.sample_seek(std::io::SeekFrom::Current(-3)).unwrap(),
            eq(0)
        );
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(0));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(0));
    }

    #[test]
    fn stereo16_read() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        let mut buf = vec![0; 8];

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![227, 34, 227, 34, 197, 34, 197, 34]));

        expect_that!(&decoder.read(&mut buf).unwrap(), eq(8));
        expect_that!(&buf, eq(vec![166, 34, 166, 34, 136, 34, 136, 34]));

        decoder.byte_seek(std::io::SeekFrom::End(-4)).unwrap();
        expect_that!(&decoder.byte_stream_position().unwrap(), eq(84920));
        expect_that!(&decoder.sample_stream_position().unwrap(), eq(21230));

        // Unable to read the whole buffer because at the end: the remaining elements
        // aren't overwritten!
        expect_that!(&decoder.read(&mut buf).unwrap(), eq(4));
        expect_that!(&buf, eq(vec![0, 0, 0, 0, 136, 34, 136, 34]));
    }

    #[test]
    #[should_panic(expected = "Invalid buffer length (7)")]
    fn stereo16_read_invalid_buffer_length() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        let mut buf = vec![0; 7];
        decoder.read(&mut buf).unwrap();
    }

    #[test]
    fn stereo16_read_to_end() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
        let content = decoder.read_to_end().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count() - 572));
    }

    #[test]
    fn stereo16_read_all() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();
        decoder.byte_seek(std::io::SeekFrom::Start(572)).unwrap();
        let content = decoder.read_all().unwrap();
        expect_that!(&content.len(), eq(decoder.byte_count()));
    }
}
