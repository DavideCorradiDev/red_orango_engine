use lazy_static::lazy_static;

pub use alto::{AltoError as AudioError, ContextAttrs as ContextDesc};

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

    pub fn default() -> Result<Self, AudioError> {
        let device = ALTO.open(None)?;
        Ok(Self { value: device })
    }

    pub fn new(device_name: &str) -> Result<Self, AudioError> {
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
    pub fn default(device: &Device) -> Result<Self, AudioError> {
        let context = device.value.new_context(None)?;
        Ok(Self { value: context })
    }

    pub fn new(device: &Device, desc: &ContextDesc) -> Result<Self, AudioError> {
        let context = device.value.new_context(Some(desc.clone()))?;
        Ok(Self { value: context })
    }
}

impl std::ops::Deref for Context {
    type Target = alto::Context;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl std::ops::DerefMut for Context {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context {{ }}")
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
}
