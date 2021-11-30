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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamingSourceDescriptor {
    pub buffer_count: u64,
    pub buffer_sample_length: u64,
}

impl std::default::Default for StreamingSourceDescriptor {
    fn default() -> Self {
        Self {
            buffer_count: 3,
            buffer_sample_length: 2048,
        }
    }
}

pub struct StreamingSource {
    value: alto::StreamingSource,
    decoder: Option<Box<dyn Decoder>>,
    buffer_sample_length: u64,
    empty_buffers: Vec<alto::Buffer>,
    looping: bool,
    processed_sample_count: u64,
    paused_sample_offset: u64,
    processing_buffer_queue: bool,
}

impl StreamingSource {
    pub fn new(context: &Context, desc: &StreamingSourceDescriptor) -> Result<Self, Error> {
        let source = context.value.new_streaming_source()?;
        let format = Format::Mono8;
        let sample_rate = 1;
        let mut empty_buffers = Vec::new();
        for _ in 0..desc.buffer_count {
            empty_buffers.push(create_buffer(
                context,
                desc.buffer_sample_length as usize * format.total_bytes_per_sample() as usize,
                format,
                sample_rate,
            )?);
        }
        Ok(Self {
            value: source,
            decoder: None,
            buffer_sample_length: desc.buffer_sample_length,
            empty_buffers,
            looping: false,
            processed_sample_count: 0,
            paused_sample_offset: 0,
            processing_buffer_queue: false,
        })
    }

    pub fn with_decoder(
        context: &Context,
        decoder: Box<dyn Decoder>,
        desc: &StreamingSourceDescriptor,
    ) -> Result<Self, Error> {
        let mut source = Self::new(context, desc)?;
        source.set_decoder(decoder)?;
        Ok(source)
    }

    // TODO: return previous decoder.
    pub fn set_decoder(&mut self, mut decoder: Box<dyn Decoder>) -> Result<(), Error> {
        self.stop();
        decoder.sample_seek(std::io::SeekFrom::Start(0))?;
        self.decoder = Some(decoder);
        self.processed_sample_count = 0;
        Ok(())
    }

    pub fn clear_decoder(&mut self) {
        self.stop();
        self.decoder = None;
        self.processed_sample_count = 0;
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
        let tbps = self.format().total_bytes_per_sample() as i32;
        assert!(processed_byte_count % tbps == 0);
        let processed_sample_count = (processed_byte_count / tbps) as u64;
        self.processed_sample_count += processed_sample_count;
        Ok(())
    }

    fn fill_buffers(&mut self) -> Result<(), Error> {
        let buffer_byte_count =
            self.buffer_sample_length as usize * self.format().total_bytes_per_sample() as usize;

        let decoder = match &mut self.decoder {
            Some(d) => d,
            None => {
                self.processing_buffer_queue = false;
                return Ok(());
            }
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
                if read_byte_count < buffer_byte_count {
                    self.processing_buffer_queue = false;
                }
                if read_byte_count == 0 {
                    return Ok(());
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

    fn set_sample_offset_internal(&mut self, value: u64) -> Result<(), Error> {
        let sample_length = self.sample_length();
        assert!(
            value < sample_length,
            "Sample offset exceeds sample length ({} >= {})",
            value,
            sample_length
        );

        self.free_buffers()?;
        self.processed_sample_count = value;
        if let Some(d) = &mut self.decoder {
            d.sample_seek(std::io::SeekFrom::Start(value))?;
        }
        self.fill_buffers()?;
        Ok(())
    }

    fn current_sample_offset(&self) -> u64 {
        let sample_length = self.sample_length();
        if sample_length == 0 {
            0
        } else {
            (self.processed_sample_count + self.value.sample_offset() as u64) % sample_length
        }
    }

    fn normalize_sample_offset(&self, value: u64) -> u64 {
        let sample_length = self.sample_length();
        if self.looping() {
            if sample_length == 0 {
                0
            } else {
                value % sample_length
            }
        } else {
            if value >= sample_length {
                0
            } else {
                value
            }
        }
    }
}

impl std::fmt::Debug for StreamingSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StreamingSource {{ }}")
    }
}

impl Source for StreamingSource {
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
        if !self.playing() && self.sample_length() > 0 {
            self.processing_buffer_queue = true;
            self.set_sample_offset_internal(self.paused_sample_offset)?;
            self.paused_sample_offset = 0;
            self.value.play();
        }
        Ok(())
    }

    fn pause(&mut self) {
        if self.playing() {
            self.processing_buffer_queue = false;
            self.value.pause();
            self.paused_sample_offset = self.current_sample_offset() as u64;
            self.value.stop();
        }
    }

    fn stop(&mut self) {
        self.processing_buffer_queue = false;
        self.value.stop();
        self.processed_sample_count = 0;
        self.paused_sample_offset = 0;
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
        if self.playing() {
            self.current_sample_offset()
        } else {
            self.paused_sample_offset
        }
    }

    fn set_sample_offset(&mut self, value: u64) -> Result<(), Error> {
        let sample_offset = self.normalize_sample_offset(value);
        if self.playing() {
            self.value.stop();
            self.set_sample_offset_internal(sample_offset)?;
            self.value.play();
        } else {
            self.paused_sample_offset = sample_offset;
        }
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
        super::{generate_source_tests, DecoderError, Device, Format},
        *,
    };
    use galvanic_assert::{matchers::*, *};

    struct DummyDecoder {
        data: Vec<u8>,
        format: Format,
        sample_rate: u32,
        byte_stream_position: u64,
    }

    impl DummyDecoder {
        fn new(format: Format, sample_count: usize, sample_rate: u32) -> Self {
            Self {
                data: vec![0; sample_count * format.total_bytes_per_sample() as usize],
                format,
                sample_rate,
                byte_stream_position: 0,
            }
        }
    }

    impl Decoder for DummyDecoder {
        fn format(&self) -> Format {
            self.format
        }

        fn sample_rate(&self) -> u32 {
            self.sample_rate
        }

        fn sample_length(&self) -> u64 {
            let tbps = self.format.total_bytes_per_sample() as usize;
            assert!(self.data.len() % tbps == 0);
            (self.data.len() / tbps) as u64
        }

        fn byte_stream_position(&mut self) -> Result<u64, DecoderError> {
            Ok(self.byte_stream_position)
        }

        fn byte_seek(&mut self, pos: std::io::SeekFrom) -> Result<u64, DecoderError> {
            let byte_length = self.byte_length() as i64;
            let target_pos = match pos {
                std::io::SeekFrom::Start(v) => v as i64,
                std::io::SeekFrom::End(v) => byte_length + v,
                std::io::SeekFrom::Current(v) => self.byte_stream_position()? as i64 + v,
            };
            let target_pos = std::cmp::max(0, std::cmp::min(target_pos, byte_length)) as u64;

            let tbps = self.format().total_bytes_per_sample() as u64;
            assert!(
                target_pos % tbps == 0,
                "Invalid seek offset ({})",
                target_pos
            );
            self.byte_stream_position = target_pos;
            Ok(target_pos)
        }

        fn read(&mut self, buf: &mut [u8]) -> Result<usize, DecoderError> {
            let tbps = self.format().total_bytes_per_sample() as usize;
            assert!(
                buf.len() % tbps == 0,
                "Invalid buffer length ({})",
                buf.len()
            );

            let leftover_byte_count = (self.byte_length() - self.byte_stream_position) as usize;
            let byte_to_read_count = std::cmp::min(buf.len(), leftover_byte_count) as usize;
            let byte_stream_position = self.byte_stream_position as usize;
            buf[0..byte_to_read_count].clone_from_slice(
                &self.data[byte_stream_position..byte_stream_position + byte_to_read_count],
            );
            self.byte_stream_position += byte_to_read_count as u64;

            Ok(byte_to_read_count)
        }
    }

    struct TestFixture {}

    impl TestFixture {
        fn create_empty(context: &Context) -> StreamingSource {
            StreamingSource::new(
                context,
                &StreamingSourceDescriptor {
                    buffer_count: 3,
                    buffer_sample_length: 32,
                },
            )
            .unwrap()
        }

        fn create_with_data(
            context: &Context,
            format: Format,
            sample_count: usize,
            sample_rate: u32,
        ) -> StreamingSource {
            StreamingSource::with_decoder(
                context,
                Box::new(DummyDecoder::new(format, sample_count, sample_rate)),
                &StreamingSourceDescriptor {
                    buffer_count: 3,
                    buffer_sample_length: sample_count as u64 / 4,
                },
            )
            .unwrap()
        }

        fn clear_data(source: &mut StreamingSource) {
            source.clear_decoder();
        }

        fn set_data(
            _context: &Context,
            source: &mut StreamingSource,
            format: Format,
            sample_count: usize,
            sample_rate: u32,
        ) {
            source
                .set_decoder(Box::new(DummyDecoder::new(
                    format,
                    sample_count,
                    sample_rate,
                )))
                .unwrap();
        }
    }

    #[test]
    #[serial_test::serial]
    fn update_buffers() {
        let device = Device::default().unwrap();
        let context = Context::default(&device).unwrap();
        let mut source = TestFixture::create_with_data(&context, Format::Stereo16, 256, 6000);
        source.play().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        expect_that!(&source.playing(), eq(true));
        source.update_buffers().unwrap();
        expect_that!(&source.sample_offset(), geq(192));
        std::thread::sleep(std::time::Duration::from_millis(50));
        expect_that!(&source.playing(), eq(false));
        expect_that!(&source.sample_offset(), eq(0));
    }

    generate_source_tests!(TestFixture);
}
