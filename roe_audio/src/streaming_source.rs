use super::{Context, Decoder, DistanceModel, Error, Format, Source};

use alto::{Mono, Source as AltoSource, SourceState, Stereo};

fn create_buffer(
    context: &Context,
    buffer_byte_length: usize,
    format: Format,
    frequency: i32,
) -> Result<alto::Buffer, Error> {
    let data = vec![0; buffer_byte_length];
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
    sample_offset: u64,
    sample_offset_override: u64,
    buffer_sample_count: u64,
    looping: bool,
    processing_buffer_queue: bool,
}

impl<D: Decoder> StreamingSource<D> {
    // TODO: descriptor.
    pub fn new(
        context: &Context,
        buffer_count: u64,
        buffer_sample_count: u64,
    ) -> Result<Self, Error> {
        let source = context.value.new_streaming_source()?;
        let format = Format::Mono8;
        let sample_rate = 1;
        let mut empty_buffers = Vec::new();
        for _ in 0..buffer_count {
            empty_buffers.push(create_buffer(
                context,
                buffer_sample_count as usize * format.total_bytes_per_sample() as usize,
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
        buffer_count: u64,
        buffer_sample_count: u64,
    ) -> Result<Self, Error> {
        let mut source = Self::new(context, buffer_count, buffer_sample_count)?;
        source.set_decoder(decoder)?;
        Ok(source)
    }

    pub fn set_decoder(&mut self, decoder: D) -> Result<(), Error> {
        self.stop();
        self.decoder = Some(decoder);
        self.set_sample_offset_var_and_stream(0)
    }

    pub fn clear_decoder(&mut self) {
        self.stop();
        self.decoder = None;
        self.sample_offset = 0;
    }

    pub fn update_buffers(&mut self) -> Result<(), Error> {
        if self.processing_buffer_queue {
            self.free_buffers()?;
            self.fill_buffers()?;

            // If self.processing_buffer_queue was true but the source is not playing, it means that
            // the buffers weren't refilled fast enough. Force the source to restart playing.
            if self.value.state() != SourceState::Playing {
                self.value.play();
            }
        }
        Ok(())
    }

    fn free_buffers(&mut self) -> Result<(), Error> {
        let mut processed_byte_count = 0;
        for _ in 0..self.value.buffers_processed() {
            let buffer = self.value.unqueue_buffer()?;
            processed_byte_count += buffer.size();
            self.empty_buffers.push(buffer);
        }
        self.sample_offset +=
            processed_byte_count as u64 / self.format().total_bytes_per_sample() as u64;
        Ok(())
    }

    fn fill_buffers(&mut self) -> Result<(), Error> {
        let buffer_byte_count =
            self.buffer_sample_count as usize * self.format().total_bytes_per_sample() as usize;

        let decoder = match &mut self.decoder {
            Some(d) => d,
            None => return Ok(()),
        };

        while self.processing_buffer_queue && self.empty_buffers.len() > 0 {
            let mut mem_buf = vec![0; buffer_byte_count];
            if self.looping {
                let mut read_byte_count = 0;
                while read_byte_count < buffer_byte_count {
                    read_byte_count += decoder.read(&mut mem_buf[read_byte_count..])?;
                    if read_byte_count < buffer_byte_count {
                        decoder.byte_seek(std::io::SeekFrom::Start(0))?;
                    }
                }
            } else {
                let read_byte_count = decoder.read(&mut mem_buf)?;
                if read_byte_count == 0 {
                    self.processing_buffer_queue = false;
                }
                mem_buf.resize(read_byte_count, 0);
            }

            let mut audio_buf = self.empty_buffers.pop().unwrap();
            set_buffer_data(
                &mut audio_buf,
                &mem_buf,
                decoder.format(),
                decoder.sample_rate() as i32,
            )?;

            self.value.queue_buffer(audio_buf)?;
        }

        Ok(())
    }

    // TODO: rename decoder counts to lengths?
    fn set_sample_offset_var_and_stream(&mut self, value: u64) -> Result<(), Error> {
        let sample_length = self.sample_length();
        assert!(
            value < sample_length,
            "Sample offset exceeds sample length ({} >= {})",
            value,
            sample_length
        );
        self.sample_offset = value;
        if let Some(d) = &mut self.decoder {
            d.sample_seek(std::io::SeekFrom::Start(value))?;
        }
        Ok(())
    }
}

impl<D: Decoder> std::fmt::Debug for StreamingSource<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StreamingSource {{ }}")
    }
}

impl<D: Decoder> Source for StreamingSource<D> {
    fn format(&self) -> Format {
        match &self.decoder {
            Some(d) => d.format(),
            None => Format::Mono8,
        }
    }

    fn sample_rate(&self) -> u32 {
        match &self.decoder {
            Some(d) => d.sample_rate(),
            None => 1,
        }
    }

    fn playing(&self) -> bool {
        self.processing_buffer_queue || self.value.state() == SourceState::Playing
    }

    fn play(&mut self) -> Result<(), Error> {
        if !self.playing() {
            self.processing_buffer_queue = true;
            self.set_sample_offset(self.sample_offset_override)?;
            self.sample_offset_override = 0;
            self.value.play();
        }
        Ok(())
    }

    fn pause(&mut self) {
        if self.playing() {
            self.processing_buffer_queue = false;
            self.value.pause();
            self.sample_offset_override = self.value.sample_offset() as u64;
            self.value.stop();
        }
    }

    fn stop(&mut self) {
        self.processing_buffer_queue = false;
        self.value.stop();
        self.sample_offset_override = 0;
    }

    fn looping(&self) -> bool {
        self.looping
    }

    fn set_looping(&mut self, value: bool) {
        self.looping = value
    }

    fn sample_length(&self) -> u64 {
        match &self.decoder {
            Some(d) => d.sample_length(),
            None => 0,
        }
    }

    fn sample_offset(&self) -> u64 {
        let sample_length = self.sample_length();
        if sample_length == 0 {
            0
        } else {
            (self.sample_offset + self.value.sample_offset() as u64) % sample_length
        }
    }

    fn set_sample_offset(&mut self, value: u64) -> Result<(), Error> {
        self.free_buffers()?;
        self.set_sample_offset_var_and_stream(value)?;
        self.fill_buffers()?;
        Ok(())
    }

    fn gain(&self) -> f32 {
        self.value.gain()
    }

    fn set_gain(&mut self, value: f32) {
        self.value.set_gain(value).unwrap()
    }

    fn min_gain(&self) -> f32 {
        self.value.min_gain()
    }

    fn set_min_gain(&mut self, value: f32) {
        self.value.set_min_gain(value).unwrap();
    }

    fn max_gain(&self) -> f32 {
        self.value.max_gain()
    }

    fn set_max_gain(&mut self, value: f32) {
        self.value.set_max_gain(value).unwrap();
    }

    fn reference_distance(&self) -> f32 {
        self.value.reference_distance()
    }

    fn set_reference_distance(&mut self, value: f32) {
        self.value.set_reference_distance(value).unwrap();
    }

    fn rolloff_factor(&self) -> f32 {
        self.value.rolloff_factor()
    }

    fn set_rolloff_factor(&mut self, value: f32) {
        self.value.set_rolloff_factor(value).unwrap();
    }

    fn max_distance(&self) -> f32 {
        self.value.max_distance()
    }

    fn set_max_distance(&mut self, value: f32) {
        self.value.set_max_distance(value).unwrap();
    }

    fn pitch(&self) -> f32 {
        self.value.pitch()
    }

    fn set_pitch(&mut self, value: f32) {
        self.value.set_pitch(value).unwrap();
    }

    fn cone_inner_angle(&self) -> f32 {
        self.value.cone_inner_angle().to_radians()
    }

    fn set_cone_inner_angle(&mut self, value: f32) {
        self.value.set_cone_inner_angle(value.to_degrees()).unwrap();
    }

    fn cone_outer_angle(&self) -> f32 {
        self.value.cone_outer_angle().to_radians()
    }

    fn set_cone_outer_angle(&mut self, value: f32) {
        self.value.set_cone_outer_angle(value.to_degrees()).unwrap();
    }

    fn cone_outer_gain(&self) -> f32 {
        self.value.cone_outer_gain()
    }

    fn set_cone_outer_gain(&mut self, value: f32) {
        self.value.set_cone_outer_gain(value).unwrap();
    }

    fn radius(&self) -> f32 {
        self.value.radius()
    }

    fn set_radius(&mut self, value: f32) {
        self.value.set_radius(value).unwrap();
    }

    fn distance_model(&self) -> DistanceModel {
        self.value.distance_model()
    }

    fn set_distance_model(&mut self, value: DistanceModel) {
        self.value.set_distance_model(value).unwrap();
    }

    fn position<V: From<[f32; 3]>>(&self) -> V {
        self.value.position()
    }

    fn set_position<V: Into<[f32; 3]>>(&mut self, value: V) {
        self.value.set_position(value).unwrap();
    }

    fn velocity<V: From<[f32; 3]>>(&self) -> V {
        self.value.velocity()
    }

    fn set_velocity<V: Into<[f32; 3]>>(&mut self, value: V) {
        self.value.set_velocity(value).unwrap();
    }

    fn direction<V: From<[f32; 3]>>(&self) -> V {
        self.value.direction()
    }

    fn set_direction<V: Into<[f32; 3]>>(&mut self, value: V) {
        self.value.set_direction(value).unwrap();
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
