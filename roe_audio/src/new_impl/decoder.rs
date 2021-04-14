use super::AudioFormat;

pub trait Decoder {
    fn audio_format(&self) -> AudioFormat;

    fn byte_rate(&self) -> u32 {
        self.sample_rate() * self.audio_format().total_bytes_per_sample()
    }

    fn packet_byte_count(&self) -> usize {
        self.packet_sample_count() * self.audio_format().total_bytes_per_sample() as usize
    }

    fn byte_count(&self) -> usize {
        self.sample_count() * self.audio_format().total_bytes_per_sample() as usize
    }

    fn byte_stream_position(&mut self) -> std::io::Result<u64> {
        Ok(self.sample_stream_position()? * self.audio_format().total_bytes_per_sample() as u64)
    }

    fn byte_seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64>;

    fn sample_rate(&self) -> u32;
    fn packet_sample_count(&self) -> usize;
    fn sample_count(&self) -> usize;
    fn sample_stream_position(&mut self) -> std::io::Result<u64>;

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

    fn read_to_end(&mut self) -> std::io::Result<Vec<u8>> {
        let tbps = self.audio_format().total_bytes_per_sample() as usize;
        let size = self.byte_count() - self.byte_stream_position()? as usize;
        assert!(size % tbps == 0);
        let mut buf = vec![0; size];
        self.read(&mut buf[..])?;
        Ok(buf)
    }

    fn read_all(&mut self) -> std::io::Result<Vec<u8>> {
        if self.byte_stream_position()? != 0 {
            self.byte_seek(std::io::SeekFrom::Start(0))?;
        }
        self.read_to_end()
    }
}
