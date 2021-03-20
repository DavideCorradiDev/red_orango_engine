use super::AudioFormat;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Sound {
    channels: Vec<Vec<u16>>,
    sample_rate: u32,
}

impl Sound {
    pub fn from_raw_data(data: &[u8], format: AudioFormat, sample_rate: u32) -> Self {
        assert!(data.len() % format.total_bytes_per_sample() as usize == 0);
        let channels = match format {
            AudioFormat::Mono8 => {
                let channel = data
                    .into_iter()
                    .map(|sample| *sample as u16 * 2u16)
                    .collect();
                vec![channel]
            }
            AudioFormat::Mono16 => {
                vec![Vec::from(bytemuck::cast_slice(data))]
            }
            AudioFormat::Stereo8 => {
                let mut left_channel =
                    Vec::with_capacity(data.len() / format.channel_count() as usize);
                let mut right_channel =
                    Vec::with_capacity(data.len() / format.channel_count() as usize);
                for i in (0..data.len()).step_by(2) {
                    left_channel.push(data[i] as u16 * 2u16);
                    right_channel.push(data[i + 1] as u16 * 2u16);
                }
                vec![left_channel, right_channel]
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
                vec![left_channel, right_channel]
            }
        };
        Self {
            channels,
            sample_rate,
        }
    }
}
