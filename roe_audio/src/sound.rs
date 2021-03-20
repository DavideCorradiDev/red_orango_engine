use super::AudioFormat;

use itertools::interleave;

fn sample8_to_sample16(sample: u8) -> i16 {
    (sample - 128) as i16 * 2
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Sound {
    left_channel: Vec<i16>,
    right_channel: Vec<i16>,
    sample_rate: u32,
}

impl Sound {
    pub fn from_raw_data(data: &[u8], format: AudioFormat, sample_rate: u32) -> Self {
        assert!(data.len() % format.total_bytes_per_sample() as usize == 0);
        let (left_channel, right_channel) = match format {
            AudioFormat::Mono8 => {
                let channel: Vec<i16> = data
                    .into_iter()
                    .map(|sample| sample8_to_sample16(*sample))
                    .collect();
                (channel.clone(), channel)
            }
            AudioFormat::Mono16 => {
                let channel = Vec::from(bytemuck::cast_slice(data));
                (channel.clone(), channel)
            }
            AudioFormat::Stereo8 => {
                let mut left_channel =
                    Vec::with_capacity(data.len() / format.channel_count() as usize);
                let mut right_channel =
                    Vec::with_capacity(data.len() / format.channel_count() as usize);
                for i in (0..data.len()).step_by(2) {
                    left_channel.push(sample8_to_sample16(data[i]));
                    right_channel.push(sample8_to_sample16(data[i + 1]));
                }
                (left_channel, right_channel)
            }
            AudioFormat::Stereo16 => {
                let data = bytemuck::cast_slice(data);
                let mut left_channel =
                    Vec::with_capacity(data.len() / format.channel_count() as usize);
                let mut right_channel =
                    Vec::with_capacity(data.len() / format.channel_count() as usize);
                for i in (0..data.len()).step_by(2) {
                    left_channel.push(data[i]);
                    right_channel.push(data[i + 1]);
                }
                (left_channel, right_channel)
            }
        };
        Self {
            left_channel,
            right_channel,
            sample_rate,
        }
    }

    pub fn left_channel(&self) -> &[i16] {
        &self.left_channel[..]
    }

    pub fn right_channel(&self) -> &[i16] {
        &self.right_channel[..]
    }

    pub fn sample_count(&self) -> usize {
        self.left_channel().len()
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn format(&self) -> AudioFormat {
        AudioFormat::Stereo16
    }

    pub fn raw_data(&self) -> Vec<u8> {
        let left_channel = Vec::from(bytemuck::cast_slice::<_, u8>(&self.left_channel[..]));
        let right_channel = Vec::from(bytemuck::cast_slice::<_, u8>(&self.right_channel[..]));
        interleave(left_channel.into_iter(), right_channel.into_iter()).collect()
    }
}
