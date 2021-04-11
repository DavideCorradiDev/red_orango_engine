use lewton::inside_ogg::OggStreamReader;

use super::{AudioFormat, Decoder};

pub use lewton::VorbisError as OggDecoderError;

pub struct OggDecoder<T: std::io::Read + std::io::Seek> {
    input: OggStreamReader<T>,
    format: AudioFormat,
    sample_count: usize,
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

    fn byte_rate(&self) -> u32 {
        self.sample_rate() * self.audio_format().total_bytes_per_sample()
    }

    fn sample_rate(&self) -> u32 {
        self.input.ident_hdr.audio_sample_rate
    }

    fn byte_count(&self) -> usize {
        self.sample_count() * self.audio_format().total_bytes_per_sample() as usize
    }

    fn sample_count(&self) -> usize {
        self.sample_count
    }

    fn byte_stream_position(&mut self) -> std::io::Result<u64> {
        Ok(0)
        // let input_pos = self.input.stream_position()?;
        // assert!(input_pos >= self.byte_data_offset);
        // Ok(input_pos - self.byte_data_offset)
    }

    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        Ok(0)
        // let byte_count = self.byte_count() as i64;
        // let target_pos = match pos {
        //     std::io::SeekFrom::Start(v) => v as i64,
        //     std::io::SeekFrom::End(v) => byte_count + v,
        //     std::io::SeekFrom::Current(v) => self.byte_stream_position()? as i64 + v,
        // };
        // let target_pos = std::cmp::max(0, std::cmp::min(target_pos, byte_count)) as u64;

        // let tbps = self.audio_format().total_bytes_per_sample() as u64;
        // assert!(
        //     target_pos % tbps == 0,
        //     "Invalid seek offset ({})",
        //     target_pos
        // );

        // let count = self
        //     .input
        //     .seek(std::io::SeekFrom::Start(self.byte_data_offset + target_pos))?;
        // Ok(count - self.byte_data_offset)
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(0)
        // let tbps = self.audio_format().total_bytes_per_sample() as usize;
        // assert!(
        //     buf.len() % tbps == 0,
        //     "Invalid buffer length ({})",
        //     buf.len()
        // );
        // let count = self.input.read(buf)?;
        // Ok(count)
    }
}
