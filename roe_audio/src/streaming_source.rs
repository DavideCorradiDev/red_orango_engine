#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StreamingSourceDescriptor {
    buffer_count: usize,
    buffer_sample_count: usize,
    looping: bool,
}

impl std::default::Default for StreamingSourceDescriptor {
    fn default() -> Self {
        Self {
            buffer_count: 3,
            buffer_sample_count: 2048,
            looping: false,
        }
    }
}

pub struct StreamingSource<D: Decoder> {
    value: alto::StreamingSource,
    decoder: D,
}

impl<D: Decoder> StreamingSource<D> {
    pub fn new(
        context: &Context,
        decoder: D,
        desc: &StreamingSourceDescriptor,
    ) -> Result<Self, BackendError> {
        let source = context.value.new_streaming_source()?;
        let mut source = Self {
            value: source,
            decoder,
        };
        // TODO: remove unwrap().
        source
            .decoder
            .byte_seek(std::io::SeekFrom::Start(0))
            .unwrap();
        let buffer_byte_count = desc.buffer_sample_count
            * source.decoder.audio_format().total_bytes_per_sample() as usize;
        for _ in 0..desc.buffer_count {
            let mut mem_buf = vec![0; buffer_byte_count];
            // TODO: remove unwrap().
            source.decoder.read(&mut mem_buf).unwrap();
            let audio_buf = buffer_with_format(
                context,
                &mem_buf,
                source.decoder.audio_format(),
                source.decoder.sample_rate() as i32,
            )?;
            source.value.queue_buffer(audio_buf)?;
        }
        Ok(source)
    }

    // TODO: make a check: if we are at the end of the decoder buffer, and the source is not looping, stop queueing stuff.
    // We should store the new buffers somewhere though, or else they will be lost...
    // Also on "play" we should make the first buffer loading...
    // TODO: implement looping as well...
    pub fn update(&mut self) -> Result<(), BackendError> {
        for _ in 0..self.value.buffers_processed() {
            let mut audio_buf = self.value.unqueue_buffer()?;
            let mut mem_buf = vec![0; audio_buf.size() as usize];
            // TODO: remove unwrap();
            let bytes_read = self.decoder.read(&mut mem_buf).unwrap();
            if bytes_read != 0 {
                set_buffer_data_with_format(
                    &mut audio_buf,
                    &mem_buf,
                    self.decoder.audio_format(),
                    self.decoder.sample_rate() as i32,
                )?;
                self.value.queue_buffer(audio_buf)?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder> std::fmt::Debug for StreamingSource<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StreamingSource {{ }}")
    }
}
