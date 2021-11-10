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
