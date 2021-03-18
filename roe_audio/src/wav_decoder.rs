use super::{AudioFormat, Decoder, DecoderError};

use bytemuck::Zeroable;

#[repr(C, packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct WavHeader {
    id: [u8; 4],
    size: u32,
    form: [u8; 4],
    chunk_id: [u8; 4],
    chunk_size: u32,
    format: u16,
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

impl WavHeader {
    fn as_slice_mut(&mut self) -> &mut [u8] {
        let h: *mut WavHeader = self;
        let h: *mut u8 = h as *mut u8;
        let data = unsafe { std::slice::from_raw_parts_mut(h, std::mem::size_of::<WavHeader>()) };
        bytemuck::cast_slice_mut(data)
    }
}

unsafe impl bytemuck::Zeroable for WavHeader {
    fn zeroed() -> Self {
        Self {
            id: [0; 4],
            size: 0,
            form: [0; 4],
            chunk_id: [0; 4],
            chunk_size: 0,
            format: 0,
            channels: 0,
            sample_rate: 0,
            byte_rate: 0,
            block_align: 0,
            bits_per_sample: 0,
        }
    }
}

unsafe impl bytemuck::Pod for WavHeader {}

pub struct WavDecoder<T: std::io::Read + std::io::Seek> {
    input: T,
    header: WavHeader,
    format: AudioFormat,
}

impl<T> WavDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    const HEADER_SIZE: usize = std::mem::size_of::<WavHeader>();

    pub fn new(mut input: T) -> Self {
        let mut header = WavHeader::zeroed();
        // TODO check channels and byte count for validity, return error otherwise.
        // TODO also check bits per sample for validity... everything.
        // TODO replace unwrap.
        input.seek(std::io::SeekFrom::Start(0)).unwrap();
        input.read_exact(header.as_slice_mut()).unwrap();
        println!("");
        println!("size of header {}", std::mem::size_of::<WavHeader>());
        println!("{:?}", header);
        println!("{:?}", header.as_slice_mut());
        let bytes_per_sample = header.bits_per_sample / 8;
        let format = AudioFormat::new(header.channels as u32, bytes_per_sample as u32);
        Self {
            input,
            header,
            format,
        }
    }
}

impl<T> Decoder for WavDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    fn audio_format(&self) -> AudioFormat {
        self.format
    }

    fn byte_stream_position(&mut self) -> Result<u64, DecoderError> {
        let input_pos = self.input.stream_position()?;
        assert!(input_pos >= Self::HEADER_SIZE as u64);
        Ok(input_pos - Self::HEADER_SIZE as u64)
    }

    fn byte_count(&self) -> usize {
        self.header.size as usize - Self::HEADER_SIZE
    }

    fn byte_rate(&self) -> u32 {
        self.header.byte_rate
    }

    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> Result<u64, DecoderError> {
        let pos = match pos {
            std::io::SeekFrom::Start(v) => std::io::SeekFrom::Start(v + Self::HEADER_SIZE as u64),
            std::io::SeekFrom::End(v) => std::io::SeekFrom::End(v + Self::HEADER_SIZE as i64),
            std::io::SeekFrom::Current(v) => {
                std::io::SeekFrom::Current(v + Self::HEADER_SIZE as i64)
            }
        };
        let count = self.input.seek(pos)?;
        Ok(count)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DecoderError> {
        let count = self.input.read(buf)?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    fn mono8_loading() {
        let file = std::fs::File::open("data/audio/mono-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let decoder = WavDecoder::new(buf);
        expect_that!(&decoder.audio_format(), eq(AudioFormat::Mono8));
        expect_that!(&decoder.byte_count(), eq(21231));
        expect_that!(&decoder.sample_count(), eq(21231));
        expect_that!(&decoder.byte_rate(), eq(44100));
        expect_that!(&decoder.sample_rate(), eq(44100));
    }
}
