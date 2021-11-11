use super::{AudioError, AudioFormat, Context, Decoder};

use alto::{Mono, Stereo};

fn create_buffer(
    context: &Context,
    buffer_byte_count: usize,
    format: AudioFormat,
    frequency: i32,
) -> Result<alto::Buffer, AudioError> {
    let data = vec![0; buffer_byte_count];
    let buffer = match format {
        AudioFormat::Mono8 => context.value.new_buffer::<Mono<u8>, _>(data, frequency),
        AudioFormat::Stereo8 => context.value.new_buffer::<Stereo<u8>, _>(data, frequency),
        AudioFormat::Mono16 => context
            .value
            .new_buffer::<Mono<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency),
        AudioFormat::Stereo16 => context
            .value
            .new_buffer::<Stereo<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency),
    }?;
    Ok(buffer)
}

fn set_buffer_data_with_format(
    buffer: &mut alto::Buffer,
    data: &[u8],
    format: AudioFormat,
    frequency: i32,
) -> Result<(), AudioError> {
    match format {
        AudioFormat::Mono8 => buffer.set_data::<Mono<u8>, _>(data, frequency),
        AudioFormat::Stereo8 => buffer.set_data::<Stereo<u8>, _>(data, frequency),
        AudioFormat::Mono16 => {
            buffer.set_data::<Mono<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency)
        }
        AudioFormat::Stereo16 => {
            buffer.set_data::<Stereo<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency)
        }
    }?;
    Ok(())
}

pub struct StreamingSource<D: Decoder> {
    value: alto::StreamingSource,
    decoder: D,
    empty_buffers: Vec<alto::Buffer>,
    buffer_byte_count: usize,
    looping: bool,
}

impl<D: Decoder> StreamingSource<D> {
    pub fn new(
        context: &Context,
        decoder: D,
    ) -> Result<Self, AudioError> {
        Self::new_with_buffer_config(context, decoder, 3, 2048)
    }

    pub fn new_with_buffer_config(
        context: &Context,
        decoder: D,
        buffer_count: usize,
        buffer_sample_count: usize,
    ) -> Result<Self, AudioError> {
        let source = context.value.new_streaming_source()?;
        let buffer_byte_count =
            buffer_sample_count * decoder.audio_format().total_bytes_per_sample() as usize;
        let mut empty_buffers = Vec::new();
        for _ in 0..buffer_count {
            empty_buffers.push(create_buffer(
                context,
                buffer_byte_count,
                decoder.audio_format(),
                decoder.sample_rate() as i32,
            )?);
        }

        let mut source = Self {
            value: source,
            decoder,
            empty_buffers,
            buffer_byte_count,
            looping: false
        };
        source.update()?;

        Ok(source)
    }

    pub fn update(&mut self) -> Result<(), AudioError> {
        // Unqueue processed buffers.
        for _ in 0..self.value.buffers_processed() {
            self.empty_buffers.push(self.value.unqueue_buffer()?);
        }

        // Read new data into empty buffers.
        let mut empty_buffer_count = self.empty_buffers.len();
        for audio_buf in self.empty_buffers.iter_mut().rev() {
            if self.looping {
                let mut read_byte_count = 0;
                let mut mem_buf = vec![0; self.buffer_byte_count];
                while read_byte_count < self.buffer_byte_count {
                    read_byte_count += self.decoder.read(&mut mem_buf[read_byte_count..])?;
                    if read_byte_count < self.buffer_byte_count {
                        self.decoder.byte_seek(std::io::SeekFrom::Start(0))?;
                    }
                }
                set_buffer_data_with_format(
                    audio_buf,
                    &mem_buf,
                    self.decoder.audio_format(),
                    self.decoder.sample_rate() as i32,
                )?;
                empty_buffer_count -= 1;
            } else {
                let mut mem_buf = vec![0; self.buffer_byte_count];
                let bytes_read = self.decoder.read(&mut mem_buf)?;
                if bytes_read == 0 {
                    break;
                }
                mem_buf.resize(bytes_read, 0);
                set_buffer_data_with_format(
                    audio_buf,
                    &mem_buf,
                    self.decoder.audio_format(),
                    self.decoder.sample_rate() as i32,
                )?;
                empty_buffer_count -= 1;
            }
        }

        // Queue populated buffers.
        let non_empty_buffers = self.empty_buffers.split_off(empty_buffer_count);
        for audio_buf in non_empty_buffers.into_iter().rev() {
            self.value.queue_buffer(audio_buf)?;
        }

        Ok(())
    }

    pub fn looping(&self) -> bool {
        self.looping
    }

    pub fn set_looping(&mut self, value: bool) {
        self.looping = value
    }
}

// TODO: substitute deref.
impl<D: Decoder> std::ops::Deref for StreamingSource<D> {
    type Target = alto::StreamingSource;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<D: Decoder> std::ops::DerefMut for StreamingSource<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<D: Decoder> std::fmt::Debug for StreamingSource<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StreamingSource {{ }}")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{Device, OggDecoder, Source, SourceState},
        *,
    };
    use galvanic_assert::{matchers::*, *};

    #[test]
    #[serial_test::serial]
    fn streaming_source_creation() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let source = StreamingSource::new(
            &context,
            OggDecoder::new(std::io::BufReader::new(
                std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap(),
            ))
            .unwrap(),
            &StreamingSourceDescriptor::default(),
        )
        .unwrap();
        expect_that!(&source.state(), eq(SourceState::Initial));
        expect_that!(&source.gain(), close_to(1., 1e-6));
        expect_that!(&source.min_gain(), close_to(0., 1e-6));
        expect_that!(&source.max_gain(), close_to(1., 1e-6));
        expect_that!(&source.reference_distance(), close_to(1., 1e-6));
        expect_that!(&source.rolloff_factor(), close_to(1., 1e-6));
        expect_that!(&source.pitch(), close_to(1., 1e-6));
    }
}
