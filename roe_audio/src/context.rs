use super::{AudioFormat, Decoder};

use lazy_static::lazy_static;

use std::sync::Arc;

pub use alto::{
    AltoError as BackendError, AsBufferData, ContextAttrs as ContextDesc, Mono, SampleFrame,
    Source, SourceState, Stereo,
};

lazy_static! {
    static ref ALTO: alto::Alto =
        alto::Alto::load_default().expect("Failed to load the audio library");
}

pub struct Device {
    value: alto::OutputDevice,
}

impl Device {
    pub fn enumerate() -> Vec<String> {
        ALTO.enumerate_outputs()
            .into_iter()
            .map(|x| x.into_string().unwrap())
            .collect()
    }

    pub fn default() -> Result<Self, BackendError> {
        let device = ALTO.open(None)?;
        Ok(Self { value: device })
    }

    pub fn new(device_name: &str) -> Result<Self, BackendError> {
        let device = ALTO.open(Some(&std::ffi::CString::new(device_name).unwrap()))?;
        Ok(Self { value: device })
    }
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Device {{ }}")
    }
}

pub struct Context {
    value: alto::Context,
}

impl Context {
    pub fn default(device: &Device) -> Result<Self, BackendError> {
        let context = device.value.new_context(None)?;
        Ok(Self { value: context })
    }

    pub fn new(device: &Device, desc: &ContextDesc) -> Result<Self, BackendError> {
        let context = device.value.new_context(Some(desc.clone()))?;
        Ok(Self { value: context })
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context {{ }}")
    }
}

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

    // TODO: rename this file to something other than context.
    // TODO: remove the "sound" class.
    // TODO: think if the Decoder should have a better interface. Or maybe separate this ufnction into another constructor accepting raw data.
    // TODO: test this function
    // TODO: must be able to propagate the errors coming from the decoder -> Need an encompassing error type.
    // TODO: test with different formats.
    // TODO: change to use buffer_With_format.
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

pub struct StaticSource {
    value: alto::StaticSource,
}

impl StaticSource {
    pub fn new(context: &Context) -> Result<Self, BackendError> {
        let static_source = context.value.new_static_source()?;
        Ok(Self {
            value: static_source,
        })
    }

    pub fn set_buffer(&mut self, buf: &Buffer) -> Result<(), BackendError> {
        self.value.set_buffer(Arc::clone(&buf.value))
    }
}

impl std::ops::Deref for StaticSource {
    type Target = alto::StaticSource;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl std::ops::DerefMut for StaticSource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl std::fmt::Debug for StaticSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StaticSource {{ }}")
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StreamingSourceDescriptor {
    buffer_count: usize,
    buffer_sample_count: usize,
    looping: bool,
}

impl std::default::Default for StreamingSourceDescriptor {
    fn default() -> Self {
        Self {
            buffer_count: 3,
            buffer_sample_count: 2048,
            looping: false,
        }
    }
}

pub struct StreamingSource<D: Decoder> {
    value: alto::StreamingSource,
    decoder: D,
}

impl<D: Decoder> StreamingSource<D> {
    pub fn new(
        context: &Context,
        decoder: D,
        desc: &StreamingSourceDescriptor,
    ) -> Result<Self, BackendError> {
        let source = context.value.new_streaming_source()?;
        let mut source = Self {
            value: source,
            decoder,
        };
        // TODO: remove unwrap().
        source
            .decoder
            .byte_seek(std::io::SeekFrom::Start(0))
            .unwrap();
        let buffer_byte_count = desc.buffer_sample_count
            * source.decoder.audio_format().total_bytes_per_sample() as usize;
        for _ in 0..desc.buffer_count {
            let mut mem_buf = vec![0; buffer_byte_count];
            // TODO: remove unwrap().
            source.decoder.read(&mut mem_buf).unwrap();
            let audio_buf = buffer_with_format(
                context,
                &mem_buf,
                source.decoder.audio_format(),
                source.decoder.sample_rate() as i32,
            )?;
            source.value.queue_buffer(audio_buf)?;
        }
        Ok(source)
    }

    pub fn update(&mut self) -> Result<(), BackendError> {
        for _ in 0..self.value.buffers_processed() {
            let mut audio_buf = self.value.unqueue_buffer()?;
            let mut mem_buf = vec![0; audio_buf.size() as usize];
            // TODO: remove unwrap();
            self.decoder.read(&mut mem_buf).unwrap();
            set_buffer_data_with_format(
                &mut audio_buf,
                &mem_buf,
                self.decoder.audio_format(),
                self.decoder.sample_rate() as i32,
            )?;
            self.value.queue_buffer(audio_buf)?;
        }
        Ok(())
    }
}

impl<D: Decoder> std::fmt::Debug for StreamingSource<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StreamingSource {{ }}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    #[serial_test::serial]
    fn enumerate_devices() {
        let devices = Device::enumerate();
        expect_that!(&devices.len(), gt(0));
    }

    #[test]
    #[serial_test::serial]
    fn default_device_creation() {
        let _ = Device::default().unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn specific_device_creation() {
        for device in Device::enumerate() {
            let _ = Device::new(&device).unwrap();
        }
    }

    #[test]
    #[serial_test::serial]
    fn default_context_creation() {
        let device = Device::default().unwrap();
        let _ = Context::default(&device).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn custom_context_creation() {
        let device = Device::default().unwrap();
        let _ = Context::new(
            &device,
            &ContextDesc {
                stereo_sources: Some(8),
                ..ContextDesc::default()
            },
        )
        .unwrap();
    }

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

    #[test]
    #[serial_test::serial]
    fn static_source_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::new::<Stereo<i16>, _>(&context, vec![0; 256], 5).unwrap();
        let mut source = StaticSource::new(&context).unwrap();
        source.set_buffer(&buffer).unwrap();
        expect_that!(&source.state(), eq(SourceState::Initial));
        expect_that!(&source.gain(), close_to(1., 1e-6));
        expect_that!(&source.min_gain(), close_to(0., 1e-6));
        expect_that!(&source.max_gain(), close_to(1., 1e-6));
        expect_that!(&source.reference_distance(), close_to(1., 1e-6));
        expect_that!(&source.rolloff_factor(), close_to(1., 1e-6));
        expect_that!(&source.pitch(), close_to(1., 1e-6));
    }
}
