use super::{Context, Decoder, Error, Format};

use alto::{Mono, Source, Stereo};

fn create_buffer(
    context: &Context,
    buffer_byte_count: usize,
    format: Format,
    frequency: i32,
) -> Result<alto::Buffer, Error> {
    let data = vec![0; buffer_byte_count];
    let buffer = match format {
        Format::Mono8 => context.value.new_buffer::<Mono<u8>, _>(data, frequency),
        Format::Stereo8 => context.value.new_buffer::<Stereo<u8>, _>(data, frequency),
        Format::Mono16 => context
            .value
            .new_buffer::<Mono<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency),
        Format::Stereo16 => context
            .value
            .new_buffer::<Stereo<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency),
    }?;
    Ok(buffer)
}

fn set_buffer_data(
    buffer: &mut alto::Buffer,
    data: &[u8],
    format: Format,
    frequency: i32,
) -> Result<(), Error> {
    match format {
        Format::Mono8 => buffer.set_data::<Mono<u8>, _>(data, frequency),
        Format::Stereo8 => buffer.set_data::<Stereo<u8>, _>(data, frequency),
        Format::Mono16 => {
            buffer.set_data::<Mono<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency)
        }
        Format::Stereo16 => {
            buffer.set_data::<Stereo<i16>, _>(bytemuck::cast_slice::<u8, i16>(&data), frequency)
        }
    }?;
    Ok(())
}

pub struct StreamingSource<D: Decoder> {
    value: alto::StreamingSource,
    decoder: Option<D>,
    empty_buffers: Vec<alto::Buffer>,
    sample_offset: usize,
    sample_offset_override: usize,
    buffer_sample_count: usize,
    looping: bool,
    processing_buffer_queue: bool,
}

impl<D: Decoder> StreamingSource<D> {
    // TODO: descriptor.
    pub fn new(
        context: &Context,
        buffer_count: usize,
        buffer_sample_count: usize,
    ) -> Result<Self, Error> {
        let source = context.value.new_streaming_source()?;
        let buffer_byte_count = 1;
        let format = Format::Mono8;
        let sample_rate = 1;
        let mut empty_buffers = Vec::new();
        for _ in 0..buffer_count {
            empty_buffers.push(create_buffer(
                context,
                buffer_sample_count * format.total_bytes_per_sample() as usize,
                format,
                sample_rate,
            )?);
        }
        Ok(Self {
            value: source,
            decoder: None,
            empty_buffers: Vec::new(),
            sample_offset: 0,
            sample_offset_override: 0,
            buffer_sample_count,
            looping: false,
            processing_buffer_queue: false,
        })
    }

    pub fn with_decoder(
        context: &Context,
        decoder: D,
        buffer_count: usize,
        buffer_sample_count: usize,
    ) -> Result<Self, Error> {
        let mut source = Self::new(context, buffer_count, buffer_sample_count)?;
        source.set_decoder(decoder);
        Ok(source)
    }

    pub fn set_decoder(&mut self, decoder: D) {
        self.stop();
        self.decoder = Some(decoder);
        self.set_stream_sample_offset(0)?;
    }

    pub fn clear_decoder(&mut self, decoder: D) {
        self.stop();
        self.decoder = None;
        self.sample_offset = 0;
    }

    fn stop(&mut self) {
        self.processing_buffer_queue = false;
        self.value.stop();
        self.sample_offset_override = 0;
    }

    fn sample_length(&self) -> usize {
        match &self.decoder {
            Some(d) => d.sample_count(),
            None => 0,
        }
    }

    // TODO: rename decoder counts to lengths?
    // TODO: replace all usizes with u64s for lengths?
    fn set_stream_sample_offset(&mut self, value: usize) -> Result<(), Error> {
        let sample_length = self.sample_length();
        assert!(
            value < sample_length,
            "Sample offset exceeds sample length ({} >= {})",
            value,
            sample_length
        );
        self.sample_offset = value;
        if let Some(d) = &mut self.decoder {
            d.sample_seek(std::io::SeekFrom::Start(value as u64))?;
        }
        Ok(())
    }

    // fn set_sample_offset_internal(&mut self, value: usize) {
    //     self.sample_offset = value;
    // }

    // pub fn new_with_buffer_config(
    //     context: &Context,
    //     decoder: D,
    //     buffer_count: usize,
    //     buffer_sample_count: usize,
    // ) -> Result<Self, Error> {
    //     let source = context.value.new_streaming_source()?;
    //     let buffer_byte_count =
    //         buffer_sample_count * decoder.format().total_bytes_per_sample() as usize;
    //     let mut empty_buffers = Vec::new();
    //     for _ in 0..buffer_count {
    //         empty_buffers.push(create_buffer(
    //             context,
    //             buffer_byte_count,
    //             decoder.format(),
    //             decoder.sample_rate() as i32,
    //         )?);
    //     }

    //     let mut source = Self {
    //         value: source,
    //         decoder,
    //         empty_buffers,
    //         buffer_byte_count,
    //         looping: false,
    //     };
    //     source.update()?;

    //     Ok(source)
    // }

    // pub fn set_decoder(&mut self, decoder: D) -> Result<(), Error> {
    //     self.value.stop();

    //     println!(
    //         "Clearing buffers (queued buffers: {}, processed buffers: {})",
    //         self.value.buffers_queued(),
    //         self.value.buffers_processed()
    //     );
    //     for _ in 0..self.value.buffers_processed() {
    //         println!("Unqueueing buffer");
    //         self.empty_buffers.push(self.value.unqueue_buffer()?);
    //     }
    //     println!(
    //         "Buffers cleared! (queued buffers: {}, processed buffers: {})",
    //         self.value.buffers_queued(),
    //         self.value.buffers_processed()
    //     );

    //     self.decoder = Some(decoder);

    //     Ok(())
    // }

    // pub fn update(&mut self) -> Result<(), Error> {
    //     let decoder = match &mut self.decoder {
    //         Some(d) => d,
    //         None => return Ok(()),
    //     };

    //     // Unqueue processed buffers.
    //     for _ in 0..self.value.buffers_processed() {
    //         self.empty_buffers.push(self.value.unqueue_buffer()?);
    //     }

    //     // TODO: simplify the following.
    //     // Read new data into empty buffers.
    //     let mut empty_buffer_count = self.empty_buffers.len();
    //     for audio_buf in self.empty_buffers.iter_mut().rev() {
    //         if self.looping {
    //             let mut mem_buf = vec![0; self.buffer_byte_count];
    //             let mut read_byte_count = 0;
    //             while read_byte_count < self.buffer_byte_count {
    //                 read_byte_count += decoder.read(&mut mem_buf[read_byte_count..])?;
    //                 if read_byte_count < self.buffer_byte_count {
    //                     decoder.byte_seek(std::io::SeekFrom::Start(0))?;
    //                 }
    //             }
    //             set_buffer_data(
    //                 audio_buf,
    //                 &mem_buf,
    //                 decoder.format(),
    //                 decoder.sample_rate() as i32,
    //             )?;
    //             empty_buffer_count -= 1;
    //         } else {
    //             let mut mem_buf = vec![0; self.buffer_byte_count];
    //             let read_byte_count = decoder.read(&mut mem_buf)?;
    //             if read_byte_count == 0 {
    //                 break;
    //             }
    //             mem_buf.resize(read_byte_count, 0);
    //             set_buffer_data(
    //                 audio_buf,
    //                 &mem_buf,
    //                 decoder.format(),
    //                 decoder.sample_rate() as i32,
    //             )?;
    //             empty_buffer_count -= 1;
    //         }
    //     }

    //     // Queue populated buffers.
    //     let non_empty_buffers = self.empty_buffers.split_off(empty_buffer_count);
    //     for audio_buf in non_empty_buffers.into_iter().rev() {
    //         self.value.queue_buffer(audio_buf)?;
    //     }

    //     Ok(())
    // }
}

impl<D: Decoder> std::fmt::Debug for StreamingSource<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StreamingSource {{ }}")
    }
}


#[cfg(test)]
mod tests {
    use super::{
        super::{Device, OggDecoder, SourceState},
        *,
    };
    use alto::Source;
    use galvanic_assert::{matchers::*, *};

    // #[test]
    // #[serial_test::serial]
    // fn dummy() {
    //     let device = Device::default().unwrap();
    //     let context = Context::default(&device).unwrap();
    //     let mut source = StreamingSource::new(&context).unwrap();
    //     source
    //         .set_decoder(
    //             &context,
    //             OggDecoder::new(std::io::BufReader::new(
    //                 std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap(),
    //             ))
    //             .unwrap(),
    //             3,
    //             2048,
    //         )
    //         .unwrap();
    //     source.value.play();
    //     source
    //         .set_decoder(
    //             &context,
    //             OggDecoder::new(std::io::BufReader::new(
    //                 std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap(),
    //             ))
    //             .unwrap(),
    //             3,
    //             2048,
    //         )
    //         .unwrap();
    // }

    // #[test]
    // #[serial_test::serial]
    // fn streaming_source_creation() {
    //     let device = Device::default().unwrap();
    //     let context = Context::default(&device).unwrap();
    //     let source = StreamingSource::new(
    //         &context,
    //         OggDecoder::new(std::io::BufReader::new(
    //             std::fs::File::open("data/audio/stereo-16-44100.ogg").unwrap(),
    //         ))
    //         .unwrap(),
    //     )
    //     .unwrap();
    //     expect_that!(&source.state(), eq(SourceState::Initial));
    //     expect_that!(&source.gain(), close_to(1., 1e-6));
    //     expect_that!(&source.min_gain(), close_to(0., 1e-6));
    //     expect_that!(&source.max_gain(), close_to(1., 1e-6));
    //     expect_that!(&source.reference_distance(), close_to(1., 1e-6));
    //     expect_that!(&source.rolloff_factor(), close_to(1., 1e-6));
    //     expect_that!(&source.pitch(), close_to(1., 1e-6));
    // }
}
