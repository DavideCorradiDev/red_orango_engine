use super::AudioFormat;

use itertools::interleave;

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
                    .map(|sample| (*sample as i16 - 128) * 2)
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
                    left_channel.push((data[i] as i16 - 128) * 2);
                    right_channel.push((data[i + 1] as i16 - 128) * 2);
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

    pub fn stereo16_raw_data(&self) -> Vec<u8> {
        let left_channel = Vec::from(bytemuck::cast_slice::<_, u8>(&self.left_channel[..]));
        let right_channel = Vec::from(bytemuck::cast_slice::<_, u8>(&self.right_channel[..]));
        interleave(left_channel.into_iter(), right_channel.into_iter()).collect()
    }
}
