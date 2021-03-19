use super::{AudioFormat, Decoder};

use bytemuck::Zeroable;

#[repr(C, packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
struct WavSignature {
    id: [u8; 4],
    size: u32,
    form: [u8; 4],
}

impl WavSignature {
    fn as_slice_mut(&mut self) -> &mut [u8] {
        let h: *mut WavSignature = self;
        let h: *mut u8 = h as *mut u8;
        let data =
            unsafe { std::slice::from_raw_parts_mut(h, std::mem::size_of::<WavSignature>()) };
        bytemuck::cast_slice_mut(data)
    }
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
#[derive(Debug, PartialEq, Clone, Copy)]
struct WavChunkSignature {
    id: [u8; 4],
    size: u32,
}

impl WavChunkSignature {
    fn as_slice_mut(&mut self) -> &mut [u8] {
        let h: *mut WavChunkSignature = self;
        let h: *mut u8 = h as *mut u8;
        let data =
            unsafe { std::slice::from_raw_parts_mut(h, std::mem::size_of::<WavChunkSignature>()) };
        bytemuck::cast_slice_mut(data)
    }
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
#[derive(Debug, PartialEq, Clone, Copy)]
struct WavFormatChunk {
    signature: WavChunkSignature,
    format: u16,
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

impl WavFormatChunk {
    fn as_slice_mut(&mut self) -> &mut [u8] {
        let h: *mut WavFormatChunk = self;
        let h: *mut u8 = h as *mut u8;
        let data =
            unsafe { std::slice::from_raw_parts_mut(h, std::mem::size_of::<WavFormatChunk>()) };
        bytemuck::cast_slice_mut(data)
    }
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
    pub fn new(mut input: T) -> Result<Self, WavDecoderError> {
        input.seek(std::io::SeekFrom::Start(0)).unwrap();

        let mut signature = WavSignature::zeroed();
        input.read_exact(signature.as_slice_mut())?;
        {
            let id_str = std::str::from_utf8(&signature.id).unwrap();
            if id_str != "RIFF" {
                return Err(WavDecoderError::InvalidId(String::from(id_str)));
            }
            let form_str = std::str::from_utf8(&signature.form).unwrap();
            if form_str != "WAVE" {
                return Err(WavDecoderError::InvalidForm(String::from(form_str)));
            }
        }

        let mut format_chunk = WavFormatChunk::zeroed();
        input.read_exact(format_chunk.as_slice_mut())?;
        {
            let id_str = std::str::from_utf8(&format_chunk.signature.id).unwrap();
            if id_str != "fmt " {
                return Err(WavDecoderError::InvalidFormatChunkId(String::from(id_str)));
            }
            if format_chunk.channels != 1 && format_chunk.channels != 2 {
                return Err(WavDecoderError::InvalidChannelCount(format_chunk.channels));
            }
            if format_chunk.bits_per_sample != 8 && format_chunk.bits_per_sample != 16 {
                return Err(WavDecoderError::InvalidBitsPerSample(
                    format_chunk.bits_per_sample,
                ));
            }
            if format_chunk.byte_rate
                != format_chunk.sample_rate
                    * format_chunk.channels as u32
                    * (format_chunk.bits_per_sample / 8) as u32
            {
                return Err(WavDecoderError::InvalidByteRate(format_chunk.byte_rate));
            }
            if format_chunk.block_align
                != format_chunk.channels * (format_chunk.bits_per_sample / 8)
            {
                return Err(WavDecoderError::InvalidBlockAlignment(
                    format_chunk.block_align,
                ));
            }
        }

        let byte_count = loop {
            let mut chunk_signature = WavChunkSignature::zeroed();
            input.read_exact(chunk_signature.as_slice_mut())?;
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

    fn byte_rate(&self) -> u32 {
        self.sample_rate * self.audio_format().total_bytes_per_sample()
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn byte_count(&self) -> usize {
        self.sample_count * self.audio_format().total_bytes_per_sample() as usize
    }

    fn sample_count(&self) -> usize {
        self.sample_count
    }

    fn byte_stream_position(&mut self) -> std::io::Result<u64> {
        let input_pos = self.input.stream_position()?;
        assert!(input_pos >= self.byte_data_offset);
        Ok(input_pos - self.byte_data_offset)
    }

    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let byte_count = self.byte_count() as i64;
        let target_pos = match pos {
            std::io::SeekFrom::Start(v) => v as i64,
            std::io::SeekFrom::End(v) => byte_count + v,
            std::io::SeekFrom::Current(v) => self.byte_stream_position()? as i64 + v,
        };
        let target_pos = std::cmp::max(0, std::cmp::min(target_pos, byte_count)) as u64;

        if target_pos % self.audio_format().total_bytes_per_sample() as u64 != 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
        }

        let count = self
            .input
            .seek(std::io::SeekFrom::Start(self.byte_data_offset + target_pos))?;
        Ok(count - self.byte_data_offset)
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let count = self.input.read(buf)?;
        Ok(count)
    }
}

#[derive(Debug)]
pub enum WavDecoderError {
    IoError(std::io::Error),
    InvalidId(String),
    InvalidForm(String),
    InvalidFormatChunkId(String),
    InvalidChannelCount(u16),
    InvalidBitsPerSample(u16),
    InvalidByteRate(u32),
    InvalidBlockAlignment(u16),
}

impl std::fmt::Display for WavDecoderError {
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

impl std::error::Error for WavDecoderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for WavDecoderError {
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
}
