use super::{AudioError, AudioFormat, Context, Decoder};

use alto::{Mono, Stereo};

use std::sync::Arc;

pub struct Buffer {
    // This member is wrapped inside an Arc because Alto::StaticSource requires so.
    // This means that the value can't be modified anymore after creation.
    // This ensures no race conditions, but also means that changing data in an existing buffer
    // is not possible: a new buffer has to be created.
    value: Arc<alto::Buffer>,
}

impl Buffer {
    pub fn new(
        context: &Context,
        data: &[u8],
        format: AudioFormat,
        frequency: i32,
    ) -> Result<Self, AudioError> {
        let buffer = match format {
            AudioFormat::Mono8 => context.value.new_buffer::<Mono<u8>, _>(data, frequency),
            AudioFormat::Stereo8 => context.value.new_buffer::<Stereo<u8>, _>(data, frequency),
            AudioFormat::Mono16 => context
                .value
                .new_buffer::<Mono<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency),
            AudioFormat::Stereo16 => context
                .value
                .new_buffer::<Stereo<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency),
        }?;
        Ok(Self {
            value: Arc::new(buffer),
        })
    }

    pub fn from_decoder<D: Decoder>(
        context: &Context,
        decoder: &mut D,
    ) -> Result<Self, AudioError> {
        let data = decoder.read_all()?;
        Self::new(
            context,
            &data,
            decoder.audio_format(),
            decoder.sample_rate() as i32,
        )
    }

    pub fn audio_format(&self) -> AudioFormat {
        let bytes_per_sample = self.value.bits() / 8;
        AudioFormat::new(self.value.channels() as u32, bytes_per_sample as u32)
    }

    pub fn byte_rate(&self) -> u32 {
        self.sample_rate() * self.audio_format().total_bytes_per_sample()
    }

    pub fn byte_count(&self) -> usize {
        self.value.size() as usize
    }

    pub fn sample_rate(&self) -> u32 {
        self.value.frequency() as u32
    }

    pub fn sample_count(&self) -> usize {
        self.byte_count() / self.audio_format().total_bytes_per_sample() as usize
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Buffer {{ }}")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{Device, WavDecoder},
        *,
    };
    use galvanic_assert::{matchers::*, *};

    #[test]
    #[serial_test::serial]
    fn mono8_buffer_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::new(
            &context,
            &[12, 13, 14, 15, 16, 17, 18, 19],
            AudioFormat::Mono8,
            5,
        )
        .unwrap();
        expect_that!(&buffer.audio_format(), eq(AudioFormat::Mono8));
        expect_that!(&buffer.byte_rate(), eq(5));
        expect_that!(&buffer.sample_rate(), eq(5));
        expect_that!(&buffer.byte_count(), eq(8));
        expect_that!(&buffer.sample_count(), eq(8));
    }

    #[test]
    #[serial_test::serial]
    fn mono16_buffer_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::new(
            &context,
            &[12, 13, 14, 15, 16, 17, 18, 19],
            AudioFormat::Mono16,
            5,
        )
        .unwrap();
        expect_that!(&buffer.audio_format(), eq(AudioFormat::Mono16));
        expect_that!(&buffer.byte_rate(), eq(10));
        expect_that!(&buffer.sample_rate(), eq(5));
        expect_that!(&buffer.byte_count(), eq(8));
        expect_that!(&buffer.sample_count(), eq(4));
    }

    #[test]
    #[serial_test::serial]
    fn stereo8_buffer_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::new(
            &context,
            &[12, 13, 14, 15, 16, 17, 18, 19],
            AudioFormat::Stereo8,
            5,
        )
        .unwrap();
        expect_that!(&buffer.audio_format(), eq(AudioFormat::Stereo8));
        expect_that!(&buffer.byte_rate(), eq(10));
        expect_that!(&buffer.sample_rate(), eq(5));
        expect_that!(&buffer.byte_count(), eq(8));
        expect_that!(&buffer.sample_count(), eq(4));
    }

    #[test]
    #[serial_test::serial]
    fn stereo16_buffer_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::new(
            &context,
            &[12, 13, 14, 15, 16, 17, 18, 19],
            AudioFormat::Stereo16,
            5,
        )
        .unwrap();
        expect_that!(&buffer.audio_format(), eq(AudioFormat::Stereo16));
        expect_that!(&buffer.byte_rate(), eq(20));
        expect_that!(&buffer.sample_rate(), eq(5));
        expect_that!(&buffer.byte_count(), eq(8));
        expect_that!(&buffer.sample_count(), eq(2));
    }

    #[test]
    #[serial_test::serial]
    fn creation_from_decoder() {
        let file = std::fs::File::open("data/audio/stereo-16-44100.wav").unwrap();
        let buf = std::io::BufReader::new(file);
        let mut decoder = WavDecoder::new(buf).unwrap();

        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::from_decoder(&context, &mut decoder).unwrap();

        expect_that!(&buffer.audio_format(), eq(AudioFormat::Stereo16));
        expect_that!(&buffer.byte_rate(), eq(44100 * 4));
        expect_that!(&buffer.sample_rate(), eq(44100));
        expect_that!(&buffer.byte_count(), eq(21231 * 4));
        expect_that!(&buffer.sample_count(), eq(21231));
    }
}
