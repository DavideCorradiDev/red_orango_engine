use super::AudioError;
use lazy_static::lazy_static;

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
    pub fn new() -> Result<Self, AudioError> {
        let device = ALTO.open(None)?;
        let context = device.new_context(None)?;
        Ok(Self { device, context })
    }

    pub fn enumerate_outputs(&self) -> Vec<String> {
        ALTO.enumerate_outputs()
            .into_iter()
            .map(|x| x.into_string().unwrap())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    fn instance_creation() {
        let instance = Instance::new().unwrap();
        println!("Outputs: {:?}.", instance.enumerate_outputs())
    }
}
