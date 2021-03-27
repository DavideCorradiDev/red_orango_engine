use super::{BackendError, Context, Sound};
use alto::Source;
use std::sync::Arc;

// TODO: Debug
pub struct Mixer {
    source: alto::StaticSource,
}

impl Mixer {
    pub fn new(context: &Context) -> Result<Self, BackendError> {
        let source = context.new_static_source()?;
        Ok(Self { source })
    }

    pub fn play(&mut self, context: &Context, sound: &Sound) -> Result<(), BackendError> {
        if self.source.state() == alto::SourceState::Playing {
            return Ok(());
        }

        let buffer = Arc::new(context.new_buffer::<alto::Stereo<i16>, _>(
            sound.interleaved_channels(),
            sound.sample_rate() as i32,
        )?);
        self.source.set_buffer(buffer)?;
        self.source.play();
        Ok(())
    }
}

impl std::fmt::Debug for Mixer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mixer {{ }}")
    }
}

// TODO: add tests
