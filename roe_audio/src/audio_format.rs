#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Mono8,
    Mono16,
    Stereo8,
    Stereo16,
}

impl AudioFormat {
    pub fn channel_count(&self) -> u32 {
        match self {
            AudioFormat::Mono8 => 1,
            AudioFormat::Mono16 => 1,
            AudioFormat::Stereo8 => 2,
            AudioFormat::Stereo16 => 2,
        }
    }

    pub fn bytes_per_sample(&self) -> u32 {
        match self {
            AudioFormat::Mono8 => 1,
            AudioFormat::Mono16 => 2,
            AudioFormat::Stereo8 => 1,
            AudioFormat::Stereo16 => 2,
        }
    }
}
