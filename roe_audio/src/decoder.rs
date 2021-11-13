use super::{Format, DecoderError};

pub trait Decoder {
    fn format(&self) -> Format;

    fn byte_rate(&self) -> u32 {
        self.sample_rate() * self.format().total_bytes_per_sample()
    }

    fn byte_count(&self) -> usize {
        self.sample_count() * self.format().total_bytes_per_sample() as usize
    }

    fn byte_stream_position(&mut self) -> Result<u64, DecoderError>;
    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> Result<u64, DecoderError>;

    fn sample_rate(&self) -> u32;
    fn sample_count(&self) -> usize;

    fn sample_stream_position(&mut self) -> Result<u64, DecoderError> {
        let byte_stream_position = self.byte_stream_position()?;
        let tbps = self.format().total_bytes_per_sample() as u64;
        assert!(byte_stream_position % tbps == 0);
        Ok(byte_stream_position / tbps)
    }

    fn sample_seek(&mut self, pos: std::io::SeekFrom) -> Result<u64, DecoderError> {
        let tbps = self.format().total_bytes_per_sample();
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
        let tbps = self.format().total_bytes_per_sample() as usize;
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
