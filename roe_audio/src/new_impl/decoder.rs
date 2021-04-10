use super::AudioFormat;

pub trait Decoder {
    fn audio_format(&self) -> AudioFormat;
    fn byte_rate(&self) -> u32;
    fn byte_count(&self) -> usize;
    fn byte_stream_position(&mut self) -> std::io::Result<u64>;
    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64>;

    fn sample_rate(&self) -> u32 {
        let tbps = self.audio_format().total_bytes_per_sample();
        assert!(self.byte_rate() % tbps == 0);
        self.byte_rate() / tbps
    }

    fn sample_count(&self) -> usize {
        let tbps = self.audio_format().total_bytes_per_sample() as usize;
        assert!(self.byte_count() % tbps == 0);
        self.byte_count() / tbps
    }

    fn sample_stream_position(&mut self) -> std::io::Result<u64> {
        let byte_pos = self.byte_stream_position()?;
        let tbps = self.audio_format().total_bytes_per_sample() as u64;
        assert!(byte_pos % tbps == 0);
        Ok(byte_pos / tbps)
    }

    fn sample_seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let tbps = self.audio_format().total_bytes_per_sample();
        let pos = match pos {
            std::io::SeekFrom::Start(v) => std::io::SeekFrom::Start(v * tbps as u64),
            std::io::SeekFrom::End(v) => std::io::SeekFrom::End(v * tbps as i64),
            std::io::SeekFrom::Current(v) => std::io::SeekFrom::Current(v * tbps as i64),
        };
        let byte_count = self.byte_seek(pos)?;
        Ok(byte_count / tbps as u64)
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()>;

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let tbps = self.audio_format().total_bytes_per_sample() as usize;
        buf.resize(self.byte_count() - self.byte_stream_position()? as usize, 0);
        assert!(buf.len() % tbps == 0);
        self.read(&mut buf[..])
    }

    fn read_all(&mut self) -> std::io::Result<Vec<u8>> {
        if self.byte_stream_position()? != 0 {
            self.byte_seek(std::io::SeekFrom::Start(0))?;
        }
        let mut vec = Vec::new();
        self.read_to_end(&mut vec)?;
        Ok(vec)
    }
}
