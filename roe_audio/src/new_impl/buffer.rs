use super::{AudioFormat, BackendError, Context, Decoder};

use alto::{Mono, Stereo};

use std::sync::Arc;

fn set_buffer_data_with_format(
    buffer: &mut alto::Buffer,
    data: &[u8],
    format: AudioFormat,
    frequency: i32,
) -> Result<(), BackendError> {
    match format {
        AudioFormat::Mono8 => buffer.set_data::<Mono<u8>, _>(data, frequency),
        AudioFormat::Stereo8 => buffer.set_data::<Stereo<u8>, _>(data, frequency),
        AudioFormat::Mono16 => {
            buffer.set_data::<Mono<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency)
        }
        AudioFormat::Stereo16 => {
            buffer.set_data::<Stereo<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency)
        }
    }?;
    Ok(())
}

pub struct Buffer {
    value: Arc<alto::Buffer>,
}

impl Buffer {
    pub fn new(
        context: &Context,
        data: &[u8],
        format: AudioFormat,
        frequency: i32,
    ) -> Result<Self, BackendError> {
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

    // TODO: unit test.
    pub fn from_decoder<D: Decoder>(
        context: &Context,
        decoder: &mut D,
    ) -> Result<Self, BackendError> {
        // TODO: replace unwrap call. Needs wrapping error type.
        let data = decoder.read_all().unwrap();
        Self::new(
            context,
            &data,
            decoder.audio_format(),
            decoder.sample_rate() as i32,
        )
    }

    // TODO: add a set_data function determining at runtime what AudioFormat to use.
    pub fn set_data(&mut self, data: &[u8], format: AudioFormat, frequency: i32) -> Result<(), BackendError> {
        match format {
            AudioFormat::Mono8 => self.value.set_data::<Mono<u8>, _>(data, frequency),
            AudioFormat::Stereo8 => self.value.set_data::<Stereo<u8>, _>(data, frequency),
            AudioFormat::Mono16 => {
                self.value.set_data::<Mono<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency)
            }
            AudioFormat::Stereo16 => {
                self.value.set_data::<Stereo<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency)
            }
        }?;
        Ok(())
    }
}

impl std::ops::Deref for Buffer {
    type Target = alto::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Buffer {{ }}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    // TODO: test with different formats.
    #[test]
    #[serial_test::serial]
    fn buffer_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::new(
            &context,
            &[12, 13, 14, 15, 16, 17, 18, 19],
            AudioFormat::Stereo16,
            5,
        )
        .unwrap();
        expect_that!(&buffer.frequency(), eq(5));
        expect_that!(&buffer.bits(), eq(16));
        expect_that!(&buffer.channels(), eq(2));
        expect_that!(&buffer.size(), eq(8));
    }
}
