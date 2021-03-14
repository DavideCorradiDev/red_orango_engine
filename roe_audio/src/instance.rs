use super::AudioError;

pub struct Instance {
    alto: alto::Alto,
}

impl Instance {
    pub fn new() -> Result<Self, AudioError> {
        let alto_lib = alto::Alto::load_default()?;
        Ok(Self { alto: alto_lib })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    fn instance_creation() {
        let _ = Instance::new().unwrap();
    }
}
