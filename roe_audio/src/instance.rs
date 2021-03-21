use lazy_static::lazy_static;

pub use alto::{AltoError as AudioError, AsBufferData, Mono, SampleFrame, Stereo};

lazy_static! {
    static ref ALTO: alto::Alto =
        alto::Alto::load_default().expect("Failed to load the audio library");
}

pub struct Instance {
    _device: alto::OutputDevice,
    context: alto::Context,
}

impl Instance {
    pub fn enumerate_devices() -> Vec<String> {
        ALTO.enumerate_outputs()
            .into_iter()
            .map(|x| x.into_string().unwrap())
            .collect()
    }

    pub fn new() -> Result<Self, AudioError> {
        let device = ALTO.open(None)?;
        let context = device.new_context(None)?;
        Ok(Self {
            _device: device,
            context,
        })
    }

    pub fn with_device(device_name: &str) -> Result<Self, AudioError> {
        let device = ALTO.open(Some(&std::ffi::CString::new(device_name).unwrap()))?;
        let context = device.new_context(None)?;
        Ok(Self {
            _device: device,
            context,
        })
    }
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Instance {{ }}")
    }
}

pub struct Buffer {
    value: alto::Buffer,
}

impl Buffer {
    pub fn new<F: SampleFrame, B: AsBufferData<F>>(
        instance: &Instance,
        data: B,
        freq: i32,
    ) -> Result<Self, AudioError> {
        let buffer = instance.context.new_buffer::<F, B>(data, freq)?;
        Ok(Self { value: buffer })
    }
}

impl std::ops::Deref for Buffer {
    type Target = alto::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl std::ops::DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Buffer {{channels: {:?}, bits: {:?}, frequency: {:?}, size: {:?}}}",
            self.value.channels(),
            self.value.bits(),
            self.value.frequency(),
            self.value.size(),
        )
    }
}

pub struct StaticSource {
    value: alto::StaticSource,
}

impl StaticSource {
    pub fn new(instance: &Instance) -> Result<Self, AudioError> {
        let source = instance.context.new_static_source()?;
        Ok(Self { value: source })
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

pub struct StreamingSource {
    value: alto::StreamingSource,
}

impl StreamingSource {
    pub fn new(instance: &Instance) -> Result<Self, AudioError> {
        let source = instance.context.new_streaming_source()?;
        Ok(Self { value: source })
    }
}

impl std::ops::Deref for StreamingSource {
    type Target = alto::StreamingSource;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl std::ops::DerefMut for StreamingSource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl std::fmt::Debug for StreamingSource {
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
    fn enumerate_outputs() {
        let devices = Instance::enumerate_devices();
        expect_that!(&devices.len(), gt(0));
    }

    #[test]
    #[serial_test::serial]
    fn default_instance_creation() {
        let _ = Instance::new().unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn specific_instance_creation() {
        for device in Instance::enumerate_devices() {
            let _ = Instance::with_device(&device).unwrap();
        }
    }

    #[test]
    #[serial_test::serial]
    fn mono8_buffer_creation() {
        let instance = Instance::new().unwrap();
        let buffer = Buffer::new::<Mono<u8>, _>(&instance, vec![0; 13], 110).unwrap();
        expect_that!(&buffer.channels(), eq(1));
        expect_that!(&buffer.bits(), eq(8));
        expect_that!(&buffer.frequency(), eq(110));
        expect_that!(&buffer.size(), eq(13));
    }

    #[test]
    #[serial_test::serial]
    fn mono16_buffer_creation() {
        let instance = Instance::new().unwrap();
        let buffer = Buffer::new::<Mono<i16>, _>(&instance, vec![0; 23], 80).unwrap();
        expect_that!(&buffer.channels(), eq(1));
        expect_that!(&buffer.bits(), eq(16));
        expect_that!(&buffer.frequency(), eq(80));
        expect_that!(&buffer.size(), eq(46));
    }

    #[test]
    #[serial_test::serial]
    fn stereo8_buffer_creation() {
        let instance = Instance::new().unwrap();
        let buffer = Buffer::new::<Stereo<u8>, _>(&instance, vec![0; 10], 123).unwrap();
        expect_that!(&buffer.channels(), eq(2));
        expect_that!(&buffer.bits(), eq(8));
        expect_that!(&buffer.frequency(), eq(123));
        expect_that!(&buffer.size(), eq(10));
    }

    #[test]
    #[serial_test::serial]
    fn stereo16_buffer_creation() {
        let instance = Instance::new().unwrap();
        let buffer = Buffer::new::<Stereo<i16>, _>(&instance, vec![0; 30], 100).unwrap();
        expect_that!(&buffer.channels(), eq(2));
        expect_that!(&buffer.bits(), eq(16));
        expect_that!(&buffer.frequency(), eq(100));
        expect_that!(&buffer.size(), eq(60));
    }

    #[test]
    #[serial_test::serial]
    fn static_source_creation() {
        let instance = Instance::new().unwrap();
        let _ = StaticSource::new(&instance).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn streaming_source_creation() {
        let instance = Instance::new().unwrap();
        let _ = StreamingSource::new(&instance).unwrap();
    }
}
