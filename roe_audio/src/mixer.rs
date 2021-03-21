use super::{AudioError, Buffer, Instance, Sound, StaticSource};

#[derive(Debug)]
pub struct Mixer {
    buffer: Option<Buffer>,
    source: StaticSource,
}

impl Mixer {
    pub fn new(instance: &Instance) -> Result<Self, AudioError> {
        let source = StaticSource::new(instance)?;
        Ok(Self {
            buffer: None,
            source,
        })
    }

    // pub fn play(&mut self, instance: &Instance, sound: &Sound) -> Result<Self, AudioError> {
    //     let mut buffer = Buffer::new(
    //         instance,
    //         &BufferDescriptor {
    //             format: sound.format(),
    //             sample_rate: sound.sample_rate(),
    //             sample_count: sound.sample_count(),
    //         },
    //     )?;
    //     buffer.set_data()
    // }
}

// TODO: add tests
