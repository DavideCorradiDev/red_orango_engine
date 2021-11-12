// TODO: rename to Format

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
            channel_count == 1 || channel_count == 2,
            "Invalid channel count ({})",
            channel_count
        );
        assert!(
            bytes_per_sample == 1 || bytes_per_sample == 2,
            "Invalid bytes per sample ({})",
            bytes_per_sample
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

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    fn new_audio_format() {
        expect_that!(&AudioFormat::new(1, 1), eq(AudioFormat::Mono8));
        expect_that!(&AudioFormat::new(1, 2), eq(AudioFormat::Mono16));
        expect_that!(&AudioFormat::new(2, 1), eq(AudioFormat::Stereo8));
        expect_that!(&AudioFormat::new(2, 2), eq(AudioFormat::Stereo16));
    }

    #[test]
    #[should_panic(expected = "Invalid channel count (0)")]
    fn invalid_channel_number() {
        let _ = AudioFormat::new(0, 1);
    }

    #[test]
    #[should_panic(expected = "Invalid bytes per sample (42)")]
    fn invalid_bytes_per_sample() {
        let _ = AudioFormat::new(1, 42);
    }

    #[test]
    fn channel_count() {
        expect_that!(&AudioFormat::Mono8.channel_count(), eq(1));
        expect_that!(&AudioFormat::Mono16.channel_count(), eq(1));
        expect_that!(&AudioFormat::Stereo8.channel_count(), eq(2));
        expect_that!(&AudioFormat::Stereo16.channel_count(), eq(2));
    }

    #[test]
    fn bytes_per_sample() {
        expect_that!(&AudioFormat::Mono8.bytes_per_sample(), eq(1));
        expect_that!(&AudioFormat::Mono16.bytes_per_sample(), eq(2));
        expect_that!(&AudioFormat::Stereo8.bytes_per_sample(), eq(1));
        expect_that!(&AudioFormat::Stereo16.bytes_per_sample(), eq(2));
    }

    #[test]
    fn total_bytes_per_sample() {
        expect_that!(&AudioFormat::Mono8.total_bytes_per_sample(), eq(1));
        expect_that!(&AudioFormat::Mono16.total_bytes_per_sample(), eq(2));
        expect_that!(&AudioFormat::Stereo8.total_bytes_per_sample(), eq(2));
        expect_that!(&AudioFormat::Stereo16.total_bytes_per_sample(), eq(4));
    }
}
