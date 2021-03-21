use lazy_static::lazy_static;

pub use alto::AltoError as AudioError;

lazy_static! {
    static ref ALTO: alto::Alto =
        alto::Alto::load_default().expect("Failed to load the audio library");
}

// TODO: should move alto to a separate struct, to allow querying devices before creating the device.
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
