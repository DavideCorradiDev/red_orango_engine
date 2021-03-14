use super::AudioError;

// TODO: should move alto to a separate struct, to allow querying devices before creating the device.
pub struct Instance {
    alto: alto::Alto,
    device: alto::OutputDevice,
    context: alto::Context,
}

impl Instance {
    pub fn new() -> Result<Self, AudioError> {
        let alto = alto::Alto::load_default()?;
        let device = alto.open(None)?;
        let context = device.new_context(None)?;
        Ok(Self {
            alto,
            device,
            context,
        })
    }

    pub fn enumerate_outputs(&self) -> Vec<String> {
        self.alto
            .enumerate_outputs()
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
