#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AudioFormat {
    Mono8,
    Mono16,
    Stereo8,
    Stereo16,
}

impl AudioFormat {
    pub fn new(channel_count: u32, bytes_per_sample: u32) -> Self {
        assert!(
            channel_count == 1
                || channel_count == 2
                || bytes_per_sample == 1
                || bytes_per_sample == 2
        );
        if channel_count == 1 {
            if bytes_per_sample == 1 {
                return Self::Mono8;
            } else if bytes_per_sample == 2 {
                return Self::Mono16;
            }
        } else if channel_count == 2 {
            if bytes_per_sample == 1 {
                return Self::Stereo8;
            } else if bytes_per_sample == 2 {
                return Self::Stereo16;
            }
        }
        unreachable!();
    }

    pub fn channel_count(&self) -> u32 {
        match self {
            Self::Mono8 => 1,
            Self::Mono16 => 1,
            Self::Stereo8 => 2,
            Self::Stereo16 => 2,
        }
    }

    pub fn bytes_per_sample(&self) -> u32 {
        match self {
            Self::Mono8 => 1,
            Self::Mono16 => 2,
            Self::Stereo8 => 1,
            Self::Stereo16 => 2,
        }
    }

    pub fn total_bytes_per_sample(&self) -> u32 {
        match self {
            Self::Mono8 => 1,
            Self::Mono16 => 2,
            Self::Stereo8 => 2,
            Self::Stereo16 => 4,
        }
    }
}
