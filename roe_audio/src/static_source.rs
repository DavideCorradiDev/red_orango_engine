use super::{AudioError, Buffer, Context};

use std::sync::Arc;

pub struct StaticSource {
    value: alto::StaticSource,
}

impl StaticSource {
    pub fn new(context: &Context) -> Result<Self, AudioError> {
        let static_source = context.value.new_static_source()?;
        Ok(Self {
            value: static_source,
        })
    }

    pub fn set_buffer(&mut self, buf: &Buffer) -> Result<(), AudioError> {
        self.value.set_buffer(Arc::clone(&buf.value))?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::{
        super::{AudioFormat, Device, Source, SourceState},
        *,
    };
    use galvanic_assert::{matchers::*, *};

    #[test]
    #[serial_test::serial]
    fn static_source_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let buffer = Buffer::new(&context, &[0; 256], AudioFormat::Stereo16, 5).unwrap();
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
