use lewton::inside_ogg::OggStreamReader;

use super::{AudioFormat, Decoder};

use lewton::OggReadError;
// TODO: rename to OggDecoderIhitializationError.
pub use lewton::VorbisError as OggDecoderError;

pub struct OggDecoder<T: std::io::Read + std::io::Seek> {
    input: OggStreamReader<T>,
    format: AudioFormat,
    sample_count: usize,
    sample_stream_position: u64,
}

impl<T> OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    pub fn new(input: T) -> Result<Self, OggDecoderError> {
        let mut input = OggStreamReader::new(input)?;
        let format = AudioFormat::new(input.ident_hdr.audio_channels as u32, 2);
        let sample_count = Self::compute_sample_count(&mut input)?;

        Ok(Self {
            input,
            format,
            sample_count,
            sample_stream_position: 0,
        })
    }

    fn compute_sample_count(input: &mut OggStreamReader<T>) -> Result<usize, OggDecoderError> {
        input.seek_absgp_pg(0)?;
        let mut sample_count = 0;
        loop {
            let data = input.read_dec_packet()?;
            match data {
                Some(data) => sample_count += data.len(),
                None => break,
            }
        }
        if input.ident_hdr.audio_channels == 2 {
            sample_count /= 2;
        }
        input.seek_absgp_pg(0)?;
        Ok(sample_count)
    }
}

impl<T> std::fmt::Debug for OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OggDecoder {{ }}")
    }
}

impl<T> Decoder for OggDecoder<T>
where
    T: std::io::Read + std::io::Seek,
{
    fn audio_format(&self) -> AudioFormat {
        self.format
    }

    fn sample_rate(&self) -> u32 {
        self.input.ident_hdr.audio_sample_rate
    }

    fn sample_count(&self) -> usize {
        self.sample_count
    }

    fn sample_stream_position(&mut self) -> std::io::Result<u64> {
        Ok(self.sample_stream_position)
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

        self.input
            .seek_absgp_pg(target_pos)
            .expect("Failed to seek ogg file");
        self.sample_stream_position = target_pos;
        Ok(self.sample_stream_position)
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut idx = 0;
        while idx < buf.len() {
            match self.input.read_dec_packet_itl() {
                Err(e) => match e {
                    OggDecoderError::OggError(e) => match e {
                        OggReadError::ReadError(e) => return Err(e),
                        _ => panic!("Unexpected error while reading from an ogg file ({})", e),
                    },
                    _ => panic!("Unexpected error while reading from an ogg file ({})", e),
                },
                Ok(v) => match v {
                    None => break,
                    Some(data) => {
                        let byte_data = bytemuck::cast_slice::<_, u8>(&data[..]);
                        let next_idx = idx + byte_data.len();
                        buf[idx..next_idx].clone_from_slice(byte_data);
                        idx = next_idx;
                    }
                },
            }
        }
        self.sample_stream_position += idx as u64;
        Ok(idx)
    }
}
