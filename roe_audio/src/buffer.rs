use super::{Context, Decoder, Error, Format};

use alto::{Mono, Stereo};

use std::sync::Arc;

pub struct Buffer {
    // This member is wrapped inside an Arc because Alto::StaticSource requires so.
    // This means that the value can't be modified anymore after creation.
    // This ensures no race conditions, but also means that changing data in an existing buffer
    // is not possible: a new buffer has to be created.
    pub(crate) value: Arc<alto::Buffer>,
}

impl Buffer {
    pub fn new(
        context: &Context,
        data: &[u8],
        format: Format,
        sample_rate: u32,
    ) -> Result<Self, Error> {
        let sample_rate = sample_rate as i32;
        let buffer = match format {
            Format::Mono8 => context.value.new_buffer::<Mono<u8>, _>(data, sample_rate),
            Format::Stereo8 => context.value.new_buffer::<Stereo<u8>, _>(data, sample_rate),
            Format::Mono16 => context
                .value
                .new_buffer::<Mono<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), sample_rate),
            Format::Stereo16 => context
                .value
                .new_buffer::<Stereo<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), sample_rate),
        }?;
        Ok(Self {
            value: Arc::new(buffer),
        })
    }

    pub fn from_decoder<D: Decoder + ?Sized>(
        context: &Context,
        decoder: &mut D,
    ) -> Result<Self, Error> {
        let data = decoder.read_all()?;
        Self::new(context, &data, decoder.format(), decoder.sample_rate())
    }

    pub fn format(&self) -> Format {
        let bytes_per_sample = self.value.bits() / 8;
        Format::new(self.value.channels() as u32, bytes_per_sample as u32)
    }

    pub fn byte_rate(&self) -> u32 {
        self.sample_rate() * self.format().total_bytes_per_sample()
    }

    pub fn byte_length(&self) -> u64 {
        self.value.size() as u64
    }

    pub fn sample_rate(&self) -> u32 {
        self.value.frequency() as u32
    }

    pub fn sample_length(&self) -> u64 {
        self.byte_length() / self.format().total_bytes_per_sample() as u64
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
            Format::Mono8,
            5,
        )
        .unwrap();
        expect_that!(&buffer.format(), eq(Format::Mono8));
        expect_that!(&buffer.byte_rate(), eq(5));
        expect_that!(&buffer.sample_rate(), eq(5));
        expect_that!(&buffer.byte_length(), eq(8));
        expect_that!(&buffer.sample_length(), eq(8));
    }

    #[test]
    #[serial_test::serial]
    fn mono16_buffer_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::new(
            &context,
            &[12, 13, 14, 15, 16, 17, 18, 19],
            Format::Mono16,
            5,
        )
        .unwrap();
        expect_that!(&buffer.format(), eq(Format::Mono16));
        expect_that!(&buffer.byte_rate(), eq(10));
        expect_that!(&buffer.sample_rate(), eq(5));
        expect_that!(&buffer.byte_length(), eq(8));
        expect_that!(&buffer.sample_length(), eq(4));
    }

    #[test]
    #[serial_test::serial]
    fn stereo8_buffer_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::new(
            &context,
            &[12, 13, 14, 15, 16, 17, 18, 19],
            Format::Stereo8,
            5,
        )
        .unwrap();
        expect_that!(&buffer.format(), eq(Format::Stereo8));
        expect_that!(&buffer.byte_rate(), eq(10));
        expect_that!(&buffer.sample_rate(), eq(5));
        expect_that!(&buffer.byte_length(), eq(8));
        expect_that!(&buffer.sample_length(), eq(4));
    }

    #[test]
    #[serial_test::serial]
    fn stereo16_buffer_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::new(
            &context,
            &[12, 13, 14, 15, 16, 17, 18, 19],
            Format::Stereo16,
            5,
        )
        .unwrap();
        expect_that!(&buffer.format(), eq(Format::Stereo16));
        expect_that!(&buffer.byte_rate(), eq(20));
        expect_that!(&buffer.sample_rate(), eq(5));
        expect_that!(&buffer.byte_length(), eq(8));
        expect_that!(&buffer.sample_length(), eq(2));
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

        expect_that!(&buffer.format(), eq(Format::Stereo16));
        expect_that!(&buffer.byte_rate(), eq(44100 * 4));
        expect_that!(&buffer.sample_rate(), eq(44100));
        expect_that!(&buffer.byte_length(), eq(21231 * 4));
        expect_that!(&buffer.sample_length(), eq(21231));
    }
}
