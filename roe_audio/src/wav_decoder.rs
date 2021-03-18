use super::{AudioFormat, Decoder};

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
            id: [0, 0, 0, 0],
            size: 0,
            form: [0, 0, 0, 0],
            chunk_id: [0, 0, 0, 0],
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
    pub fn new(mut input: T) -> Self {
        let mut header = WavHeader::zeroed();
        // TODO check channels and byte count for validity, return error otherwise.
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

// impl<T> Decoder for WavDecoder<T>
// where
//     T: std::io::Read + std::io::Seek,
// {
//     fn audio_format(&self) -> AudioFormat {
//         self.format
//     }
//
//     fn sample_rate(&self) -> u32 {
//         self.header.sample_rate
//     }
//
//     fn sample_count(&self) -> usize {
//         self.header.sample_count
//     }
//
//     fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize>;
//     fn read(&mut self, bug: &mut [u8]) -> std::io::Result<usize>;
// }

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    fn load_mono8() {
        let file = std::fs::File::open("data/audio/mono-8-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let _ = WavDecoder::new(buf);
    }
}
