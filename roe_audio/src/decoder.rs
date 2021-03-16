use super::AudioFormat;

pub trait Decoder<T: std::io::Read + std::io::Seek>: std::io::Seek {
    fn audio_format(&self) -> AudioFormat;
    fn sample_rate(&self) -> u32;
    fn sample_count(&self) -> usize;
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize>;
    fn read(&mut self, bug: &mut [u8]) -> std::io::Result<usize>;
}
