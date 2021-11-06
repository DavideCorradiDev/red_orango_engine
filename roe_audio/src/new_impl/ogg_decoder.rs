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
    pub fn new(input: T) -> Self {
        let mut decoder = vorbis::Decoder::new(input).unwrap();
        let mut channel_count = 0;
        let mut sample_rate = 0;
        let mut sample_count = 0;
        for packet in decoder.packets() {
            let packet = packet.unwrap();
            channel_count = packet.channels as u32;
            sample_rate = packet.rate as u32;
            sample_count += packet.data.len();
        }
        assert!(channel_count == 1 || channel_count == 2);
        assert!(sample_count % channel_count as usize == 0);
        sample_count /= channel_count as usize;

        const BYTES_PER_SAMPLE: u32 = 2;
        let format = AudioFormat::new(channel_count, BYTES_PER_SAMPLE);

        Self {
            decoder,
            format,
            sample_rate,
            sample_count
        }
    }
}
