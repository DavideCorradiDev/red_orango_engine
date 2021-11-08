use super::{AudioFormat, BackendError, Context, Device, Decoder};

pub use alto::{AsBufferData, Mono, SampleFrame, Stereo};

use std::sync::Arc;

fn buffer_with_format(
    context: &Context,
    data: &[u8],
    format: AudioFormat,
    frequency: i32,
) -> Result<alto::Buffer, BackendError> {
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
    Ok(buffer)
}

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
    pub fn new<F: SampleFrame, B: AsBufferData<F>>(
        context: &Context,
        data: B,
        frequency: i32,
    ) -> Result<Self, BackendError> {
        let buffer = context.value.new_buffer::<F, B>(data, frequency)?;
        Ok(Self {
            value: Arc::new(buffer),
        })
    }

    // TODO: test this function
    // TODO: must be able to propagate the errors coming from the decoder -> Need an encompassing error type.
    // TODO: test with different formats.
    // TODO: change to use buffer_with_format.
    pub fn from_decoder<D: Decoder>(
        context: &Context,
        decoder: &mut D,
    ) -> Result<Self, BackendError> {
        // TODO: replace unwrap call.
        let data = decoder.read_all().unwrap();
        let buffer = match decoder.audio_format() {
            AudioFormat::Mono8 => {
                Self::new::<Mono<u8>, _>(context, data, decoder.sample_rate() as i32)
            }
            AudioFormat::Stereo8 => {
                Self::new::<Stereo<u8>, _>(context, data, decoder.sample_rate() as i32)
            }
            AudioFormat::Mono16 => Self::new::<Mono<i16>, _>(
                context,
                bytemuck::cast_slice::<u8, i16>(&data),
                decoder.sample_rate() as i32,
            ),
            AudioFormat::Stereo16 => Self::new::<Stereo<i16>, _>(
                context,
                bytemuck::cast_slice::<u8, i16>(&data),
                decoder.sample_rate() as i32,
            ),
        }?;
        Ok(buffer)
    }

    // TODO: add a set_data function determining at runtime what AudioFormat to use.
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

    #[test]
    #[serial_test::serial]
    fn buffer_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::new::<Stereo<i16>, _>(&context, vec![12, 13, 14, 15], 5).unwrap();
        expect_that!(&buffer.frequency(), eq(5));
        expect_that!(&buffer.bits(), eq(16));
        expect_that!(&buffer.channels(), eq(2));
        expect_that!(&buffer.size(), eq(8));
    }
}