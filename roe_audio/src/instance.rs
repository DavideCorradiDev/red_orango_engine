use lazy_static::lazy_static;

use super::AudioFormat;

pub use alto::AltoError as AudioError;

lazy_static! {
    static ref ALTO: alto::Alto =
        alto::Alto::load_default().expect("Failed to load the audio library");
}

// TODO: should move alto to a separate struct, to allow querying devices before creating the device.
// TODO: add Debug derivation.
pub struct Instance {
    device: alto::OutputDevice,
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
        Ok(Self { device, context })
    }

    pub fn with_device(device_name: &str) -> Result<Self, AudioError> {
        let device = ALTO.open(Some(&std::ffi::CString::new(device_name).unwrap()))?;
        let context = device.new_context(None)?;
        Ok(Self { device, context })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BufferDescriptor {
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub sample_count: usize,
}

// TODO: derive debug.
pub struct Buffer {
    value: alto::Buffer,
}

impl Buffer {
    pub fn new(instance: &Instance, desc: &BufferDescriptor) -> Result<Self, AudioError> {
        let buffer = match desc.format {
            AudioFormat::Mono8 => {
                let mut dummy_data = Vec::new();
                dummy_data.resize(desc.sample_count, 0);
                instance.context.new_buffer::<alto::Mono<u8>, _>(
                    dummy_data.as_slice(),
                    desc.sample_rate as i32,
                )?
            }
            AudioFormat::Mono16 => {
                let mut dummy_data = Vec::new();
                dummy_data.resize(desc.sample_count, 0);
                instance.context.new_buffer::<alto::Mono<i16>, _>(
                    dummy_data.as_slice(),
                    desc.sample_rate as i32,
                )?
            }
            AudioFormat::Stereo8 => {
                let mut dummy_data = Vec::new();
                dummy_data.resize(desc.sample_count * 2, 0);
                instance.context.new_buffer::<alto::Stereo<u8>, _>(
                    dummy_data.as_slice(),
                    desc.sample_rate as i32,
                )?
            }
            AudioFormat::Stereo16 => {
                let mut dummy_data = Vec::new();
                dummy_data.resize(desc.sample_count * 2, 0);
                instance.context.new_buffer::<alto::Stereo<i16>, _>(
                    dummy_data.as_slice(),
                    desc.sample_rate as i32,
                )?
            }
        };
        Ok(Self { value: buffer })
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
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Instance {{ }}")
    }
}
