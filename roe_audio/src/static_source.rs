use super::{Buffer, Context, DistanceModel, Error, Format, Source};

use alto::Source as AltoSource;

use std::sync::Arc;

pub struct StaticSource {
    value: alto::StaticSource,
    format: Format,
    sample_length: usize,
    sample_rate: u32,
    // Variable used to ensure consistency when retrieving the current sample offset when the source
    // is not playing.
    sample_offset_override: usize,
}

impl StaticSource {
    const DEFAULT_AUDIO_FORMAT: Format = Format::Mono8;
    const DEFAULT_SAMPLE_RATE: u32 = 1;

    pub fn new(context: &Context) -> Result<Self, Error> {
        let static_source = context.value.new_static_source()?;
        Ok(Self {
            value: static_source,
            format: Self::DEFAULT_AUDIO_FORMAT,
            sample_length: 0,
            sample_rate: Self::DEFAULT_SAMPLE_RATE,
            sample_offset_override: 0,
        })
    }
    pub fn with_buffer(context: &Context, buf: &Buffer) -> Result<Self, Error> {
        let mut static_source = Self::new(context)?;
        static_source.set_buffer(buf)?;
        Ok(static_source)
    }

    pub fn set_buffer(&mut self, buf: &Buffer) -> Result<(), Error> {
        self.value.stop();
        self.value.set_buffer(Arc::clone(&buf.value))?;
        self.format = buf.format();
        self.sample_length = buf.sample_count();
        self.sample_rate = buf.sample_rate();
        self.sample_offset_override = 0;
        Ok(())
    }

    pub fn clear_buffer(&mut self) {
        self.value.stop();
        self.value.clear_buffer();
        self.format = Self::DEFAULT_AUDIO_FORMAT;
        self.sample_length = 0;
        self.sample_rate = Self::DEFAULT_SAMPLE_RATE;
        self.sample_offset_override = 0;
    }
}

impl Source for StaticSource {
    fn format(&self) -> Format {
        self.format
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn playing(&self) -> bool {
        self.value.state() == alto::SourceState::Playing
    }

    fn play(&mut self) -> Result<(), Error> {
        if !self.playing() {
            // Update to requested sample offset.
            self.value
                .set_sample_offset(self.sample_offset_override as i32)?;
            // Set requested sample offset to 0 in case playback ends on its own.
            self.sample_offset_override = 0;
            self.value.play();
        }
        Ok(())
    }

    fn pause(&mut self) {
        if self.playing() {
            // Pause and save current offset.
            self.value.pause();
            self.sample_offset_override = self.value.sample_offset() as usize;
            // Actually stop the source to reduce the number of states to be managed.
            self.value.stop();
        }
    }

    fn stop(&mut self) {
        self.value.stop();
        self.sample_offset_override = 0;
    }

    fn looping(&self) -> bool {
        self.value.looping()
    }

    fn set_looping(&mut self, value: bool) {
        self.value.set_looping(value)
    }

    fn sample_length(&self) -> usize {
        self.sample_length
    }

    fn sample_offset(&self) -> usize {
        if self.playing() {
            self.value.sample_offset() as usize
        } else {
            self.sample_offset_override
        }
    }

    fn set_sample_offset(&mut self, value: usize) -> Result<(), Error> {
        assert!(
            value < self.sample_length(),
            "Sample offset exceeds sample length ({} >= {})",
            value,
            self.sample_length()
        );
        if self.playing() {
            // If currently playing, stop, set offset, and resume.
            self.value.stop();
            self.value.set_sample_offset(value as alto::sys::ALint)?;
            self.value.play();
        } else {
            // If not currently playing, store the requested offset.
            self.sample_offset_override = std::cmp::min(value, self.sample_length());
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

    fn cone_inner_angle(&self) -> f32 {
        self.value.cone_inner_angle()
    }

    fn set_cone_inner_angle(&mut self, value: f32) {
        self.value.set_cone_inner_angle(value).unwrap();
    }

    fn cone_outer_angle(&self) -> f32 {
        self.value.cone_outer_angle()
    }

    fn set_cone_outer_angle(&mut self, value: f32) {
        self.value.set_cone_outer_angle(value).unwrap();
    }

    fn cone_outer_gain(&self) -> f32 {
        self.value.cone_outer_gain()
    }

    fn set_cone_outer_gain(&mut self, value: f32) {
        self.value.set_cone_outer_gain(value).unwrap();
    }

    fn distance_model(&self) -> DistanceModel {
        self.value.distance_model()
    }

    fn set_distance_model(&mut self, value: DistanceModel) {
        self.value.set_distance_model(value).unwrap();
    }

    fn radius(&self) -> f32 {
        self.value.radius()
    }

    fn set_radius(&self, value: f32) {
        self.value.set_radius(value).unwrap();
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
        super::{Device, Format},
        *,
    };
    use galvanic_assert::{matchers::*, *};

    fn create_context() -> Context {
        let device = Device::default().unwrap();
        Context::default(&device).unwrap()
    }

    fn create_source() -> StaticSource {
        let context = create_context();
        let buf = Buffer::new(&context, &[0; 256], Format::Stereo16, 100).unwrap();
        StaticSource::with_buffer(&context, &buf).unwrap()
    }

    // TODO: test individual properties with setters / getters.

    // Creation tests

    #[test]
    #[serial_test::serial]
    fn creation() {
        let context = create_context();
        let source = StaticSource::new(&context).unwrap();

        expect_that!(&source.format(), eq(Format::Mono8));
        expect_that!(&source.sample_rate(), eq(1));
        expect_that!(&source.playing(), eq(false));
        expect_that!(&source.looping(), eq(false));
        expect_that!(&source.sample_length(), eq(0));
        expect_that!(&source.sample_offset(), eq(0));
        expect_that!(&source.byte_length(), eq(0));
        expect_that!(&source.byte_offset(), eq(0));
        expect_that!(&source.time_length().as_secs_f64(), close_to(0., 1e-6));
        expect_that!(&source.time_offset().as_secs_f64(), close_to(0., 1e-6));

        expect_that!(&source.gain(), close_to(1., 1e-6));
        expect_that!(&source.min_gain(), close_to(0., 1e-6));
        expect_that!(&source.max_gain(), close_to(1., 1e-6));
        expect_that!(&source.reference_distance(), close_to(1., 1e-6));
        expect_that!(&source.rolloff_factor(), close_to(1., 1e-6));
        expect_that!(&source.pitch(), close_to(1., 1e-6));
        expect_that!(&source.position(), eq([0., 0., 0.]));
        expect_that!(&source.velocity(), eq([0., 0., 0.]));
        expect_that!(&source.direction(), eq([0., 0., 0.]));
        expect_that!(&source.cone_inner_angle(), close_to(360., 1e-6));
        expect_that!(&source.cone_outer_angle(), close_to(360., 1e-6));
        expect_that!(&source.cone_outer_gain(), close_to(0., 1e-6));
        expect_that!(&source.distance_model(), eq(DistanceModel::InverseClamped));
        expect_that!(&source.radius(), close_to(0., 1e-6));
    }

    #[test]
    #[serial_test::serial]
    fn creation_with_buffer() {
        let context = create_context();
        let buf = Buffer::new(&context, &[0; 256], Format::Stereo16, 10).unwrap();
        let source = StaticSource::with_buffer(&context, &buf).unwrap();

        expect_that!(&source.format(), eq(Format::Stereo16));
        expect_that!(&source.sample_rate(), eq(10));
        expect_that!(&source.playing(), eq(false));
        expect_that!(&source.looping(), eq(false));
        expect_that!(&source.sample_length(), eq(64));
        expect_that!(&source.sample_offset(), eq(0));
        expect_that!(&source.byte_length(), eq(256));
        expect_that!(&source.byte_offset(), eq(0));
        expect_that!(&source.time_length().as_secs_f64(), close_to(6.4, 1e-6));
        expect_that!(&source.time_offset().as_secs_f64(), close_to(0., 1e-6));

        expect_that!(&source.gain(), close_to(1., 1e-6));
        expect_that!(&source.min_gain(), close_to(0., 1e-6));
        expect_that!(&source.max_gain(), close_to(1., 1e-6));
        expect_that!(&source.reference_distance(), close_to(1., 1e-6));
        expect_that!(&source.rolloff_factor(), close_to(1., 1e-6));
        expect_that!(&source.pitch(), close_to(1., 1e-6));
        expect_that!(&source.position(), eq([0., 0., 0.]));
        expect_that!(&source.velocity(), eq([0., 0., 0.]));
        expect_that!(&source.direction(), eq([0., 0., 0.]));
        expect_that!(&source.cone_inner_angle(), close_to(360., 1e-6));
        expect_that!(&source.cone_outer_angle(), close_to(360., 1e-6));
        expect_that!(&source.cone_outer_gain(), close_to(0., 1e-6));
        expect_that!(&source.distance_model(), eq(DistanceModel::InverseClamped));
        expect_that!(&source.radius(), close_to(0., 1e-6));
    }

    #[test]
    #[serial_test::serial]
    fn clear_buffer() {
        let context = create_context();
        let buf = Buffer::new(&context, &[0; 256], Format::Stereo16, 10).unwrap();
        let mut source = StaticSource::with_buffer(&context, &buf).unwrap();
        source.clear_buffer();

        expect_that!(&source.format(), eq(Format::Mono8));
        expect_that!(&source.sample_rate(), eq(1));
        expect_that!(&source.playing(), eq(false));
        expect_that!(&source.looping(), eq(false));
        expect_that!(&source.sample_length(), eq(0));
        expect_that!(&source.sample_offset(), eq(0));
        expect_that!(&source.byte_length(), eq(0));
        expect_that!(&source.byte_offset(), eq(0));
        expect_that!(&source.time_length().as_secs_f64(), close_to(0., 1e-6));
        expect_that!(&source.time_offset().as_secs_f64(), close_to(0., 1e-6));

        expect_that!(&source.gain(), close_to(1., 1e-6));
        expect_that!(&source.min_gain(), close_to(0., 1e-6));
        expect_that!(&source.max_gain(), close_to(1., 1e-6));
        expect_that!(&source.reference_distance(), close_to(1., 1e-6));
        expect_that!(&source.rolloff_factor(), close_to(1., 1e-6));
        expect_that!(&source.pitch(), close_to(1., 1e-6));
        expect_that!(&source.position(), eq([0., 0., 0.]));
        expect_that!(&source.velocity(), eq([0., 0., 0.]));
        expect_that!(&source.direction(), eq([0., 0., 0.]));
        expect_that!(&source.cone_inner_angle(), close_to(360., 1e-6));
        expect_that!(&source.cone_outer_angle(), close_to(360., 1e-6));
        expect_that!(&source.cone_outer_gain(), close_to(0., 1e-6));
        expect_that!(&source.distance_model(), eq(DistanceModel::InverseClamped));
        expect_that!(&source.radius(), close_to(0., 1e-6));
    }

    #[test]
    #[serial_test::serial]
    fn set_buffer() {
        let context = create_context();
        let mut source = StaticSource::new(&context).unwrap();
        let buf = Buffer::new(&context, &[0; 256], Format::Stereo16, 10).unwrap();
        source.set_buffer(&buf).unwrap();

        expect_that!(&source.format(), eq(Format::Stereo16));
        expect_that!(&source.sample_rate(), eq(10));
        expect_that!(&source.playing(), eq(false));
        expect_that!(&source.looping(), eq(false));
        expect_that!(&source.sample_length(), eq(64));
        expect_that!(&source.sample_offset(), eq(0));
        expect_that!(&source.byte_length(), eq(256));
        expect_that!(&source.byte_offset(), eq(0));
        expect_that!(&source.time_length().as_secs_f64(), close_to(6.4, 1e-6));
        expect_that!(&source.time_offset().as_secs_f64(), close_to(0., 1e-6));

        expect_that!(&source.gain(), close_to(1., 1e-6));
        expect_that!(&source.min_gain(), close_to(0., 1e-6));
        expect_that!(&source.max_gain(), close_to(1., 1e-6));
        expect_that!(&source.reference_distance(), close_to(1., 1e-6));
        expect_that!(&source.rolloff_factor(), close_to(1., 1e-6));
        expect_that!(&source.pitch(), close_to(1., 1e-6));
        expect_that!(&source.position(), eq([0., 0., 0.]));
        expect_that!(&source.velocity(), eq([0., 0., 0.]));
        expect_that!(&source.direction(), eq([0., 0., 0.]));
        expect_that!(&source.cone_inner_angle(), close_to(360., 1e-6));
        expect_that!(&source.cone_outer_angle(), close_to(360., 1e-6));
        expect_that!(&source.cone_outer_gain(), close_to(0., 1e-6));
        expect_that!(&source.distance_model(), eq(DistanceModel::InverseClamped));
        expect_that!(&source.radius(), close_to(0., 1e-6));
    }

    // Offset tests

    #[test]
    #[serial_test::serial]
    fn set_sample_offset_while_paused() {
        let context = create_context();
        let buf = Buffer::new(&context, &[0; 256], Format::Stereo16, 100).unwrap();
        let mut source = StaticSource::with_buffer(&context, &buf).unwrap();

        expect_that!(&source.sample_offset(), eq(0));
        source.set_sample_offset(24).unwrap();
        expect_that!(&source.sample_offset(), eq(24));
    }

    #[test]
    #[serial_test::serial]
    fn set_sample_offset_while_playing() {
        let mut source = create_source();
        expect_that!(&source.sample_offset(), eq(0));
        source.play().unwrap();
        expect_that!(&source.sample_offset(), not(geq(24)));
        source.set_sample_offset(24).unwrap();
        expect_that!(&source.sample_offset(), geq(24));
    }

    #[test]
    #[serial_test::serial]
    fn get_sample_offset_after_play() {
        let mut source = create_source();
        source.set_sample_offset(24).unwrap();
        expect_that!(&source.sample_offset(), eq(24));
        source.play().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        expect_that!(&source.sample_offset(), gt(24));
    }

    #[test]
    #[serial_test::serial]
    fn get_sample_offset_after_pause() {
        let mut source = create_source();
        source.set_sample_offset(24).unwrap();
        expect_that!(&source.sample_offset(), eq(24));
        source.play().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        source.pause();
        expect_that!(&source.sample_offset(), gt(24));
    }

    #[test]
    #[serial_test::serial]
    fn get_sample_offset_after_stop() {
        let mut source = create_source();
        source.set_sample_offset(24).unwrap();
        expect_that!(&source.sample_offset(), eq(24));
        source.play().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        source.stop();
        expect_that!(&source.sample_offset(), eq(0));
    }

    #[test]
    #[serial_test::serial]
    fn get_sample_offset_after_pause_and_stop() {
        let mut source = create_source();
        source.set_sample_offset(24).unwrap();
        expect_that!(&source.sample_offset(), eq(24));
        source.play().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        source.pause();
        source.stop();
        expect_that!(&source.sample_offset(), eq(0));
    }

    #[test]
    #[serial_test::serial]
    fn get_sample_offset_after_several_pauses() {
        let mut source = create_source();
        let pos0 = source.sample_offset();

        source.play().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        source.pause();
        let pos1 = source.sample_offset();

        source.play().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        source.pause();
        let pos2 = source.sample_offset();

        source.play().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        source.pause();
        let pos3 = source.sample_offset();

        expect_that!(&pos1, gt(pos0));
        expect_that!(&pos2, gt(pos1));
        expect_that!(&pos3, gt(pos2));
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "Sample offset exceeds sample length (100 >= 64)")]
    fn set_sample_offset_exceeds_sample_length() {
        let mut source = create_source();
        source.set_sample_offset(100).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn set_time_offset() {
        let mut source = create_source();
        expect_that!(&source.time_offset().as_secs_f64(), close_to(0., 1e-6));

        source
            .set_time_offset(std::time::Duration::from_secs_f64(0.33))
            .unwrap();
        expect_that!(&source.time_offset().as_secs_f64(), close_to(0.33, 1e-6));

        source
            .set_time_offset(std::time::Duration::from_secs_f64(0.))
            .unwrap();
        expect_that!(&source.time_offset().as_secs_f64(), close_to(0., 1e-6));
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "underflow when converting float to duration")]
    fn set_time_offset_negative() {
        let mut source = create_source();
        source
            .set_time_offset(std::time::Duration::from_secs_f64(-1.))
            .unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn set_byte_offset() {
        let mut source = create_source();
        expect_that!(&source.byte_length(), eq(256));
        expect_that!(&source.byte_offset(), eq(0));

        source.set_byte_offset(24).unwrap();
        expect_that!(&source.byte_offset(), eq(24));

        source.set_byte_offset(0).unwrap();
        expect_that!(&source.byte_offset(), eq(0));
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "Byte offset is within sample (3)")]
    fn set_byte_offset_within_sample() {
        let mut source = create_source();
        source.set_byte_offset(3).unwrap();
    }

    // Playback tests

    #[test]
    #[serial_test::serial]
    fn play_at_initial_state() {
        let mut source = create_source();
        let pos0 = source.sample_offset();

        source.play().unwrap();
        let pos1 = source.sample_offset();
        expect_that!(&source.playing(), eq(true));
        
        expect_that!(&pos1, geq(pos0));
    }

    #[test]
    #[serial_test::serial]
    fn play_after_play() {
        let mut source = create_source();
        let pos0 = source.sample_offset();

        source.play().unwrap();
        let pos1 = source.sample_offset();
        expect_that!(&source.playing(), eq(true));

        source.play().unwrap();
        let pos2 = source.sample_offset();
        expect_that!(&source.playing(), eq(true));

        expect_that!(&pos1, geq(pos0));
        expect_that!(&pos2, geq(pos1));
    }

    #[test]
    #[serial_test::serial]
    fn play_after_pause() {
        let mut source = create_source();
        let pos0 = source.sample_offset();

        source.play().unwrap();
        let pos1 = source.sample_offset();
        expect_that!(&source.playing(), eq(true));

        source.pause();
        let pos2 = source.sample_offset();
        expect_that!(&source.playing(), eq(false));

        source.play().unwrap();
        let pos3 = source.sample_offset();
        expect_that!(&source.playing(), eq(true));

        expect_that!(&pos1, geq(pos0));
        expect_that!(&pos2, geq(pos1));
        expect_that!(&pos3, geq(pos2));
    }

    #[test]
    #[serial_test::serial]
    fn play_after_stop() {
        let mut source = create_source();
        let pos0 = source.sample_offset();

        source.play().unwrap();
        let pos1 = source.sample_offset();
        expect_that!(&source.playing(), eq(true));

        source.stop();
        let pos2 = source.sample_offset();
        expect_that!(&source.playing(), eq(false));

        source.play().unwrap();
        let pos3 = source.sample_offset();
        expect_that!(&source.playing(), eq(true));
        
        expect_that!(&pos1, geq(pos0));
        expect_that!(&pos2, eq(0));
        expect_that!(&pos3, geq(pos2));
    }

    #[test]
    #[serial_test::serial]
    fn pause_at_initial_state() {
        let mut source = create_source();
        source.set_sample_offset(24).unwrap();
        let pos0 = source.sample_offset();

        source.pause();
        let pos1 = source.sample_offset();
        expect_that!(&source.playing(), eq(false));
        
        expect_that!(&pos1, eq(pos0));
    }

    #[test]
    #[serial_test::serial]
    fn pause_after_play() {
        let mut source = create_source();
        let pos0 = source.sample_offset();

        source.play().unwrap();
        let pos1 = source.sample_offset();
        expect_that!(&source.playing(), eq(true));

        source.pause();
        let pos2 = source.sample_offset();
        expect_that!(&source.playing(), eq(false));

        expect_that!(&pos1, geq(pos0));
        expect_that!(&pos2, geq(pos1));
    }

    #[test]
    #[serial_test::serial]
    fn pause_after_pause() {
        let mut source = create_source();
        let pos0 = source.sample_offset();

        source.play().unwrap();
        let pos1 = source.sample_offset();
        expect_that!(&source.playing(), eq(true));

        source.pause();
        let pos2 = source.sample_offset();
        expect_that!(&source.playing(), eq(false));

        source.pause();
        let pos3 = source.sample_offset();
        expect_that!(&source.playing(), eq(false));

        expect_that!(&pos1, geq(pos0));
        expect_that!(&pos2, geq(pos1));
        expect_that!(&pos3, eq(pos2));
    }

    #[test]
    #[serial_test::serial]
    fn pause_after_stop() {
        let mut source = create_source();
        let pos0 = source.sample_offset();

        source.play().unwrap();
        let pos1 = source.sample_offset();
        expect_that!(&source.playing(), eq(true));

        source.stop();
        let pos2 = source.sample_offset();
        expect_that!(&source.playing(), eq(false));

        source.pause();
        let pos3 = source.sample_offset();
        expect_that!(&source.playing(), eq(false));

        expect_that!(&pos1, geq(pos0));
        expect_that!(&pos2, eq(0));
        expect_that!(&pos3, eq(0));
    }

    // Properties tests

    #[test]
    #[serial_test::serial]
    fn looping() {
        let mut source = create_source();
        expect_that!(&source.looping(), eq(false));

        source.set_looping(true);
        expect_that!(&source.looping(), eq(true));

        source.set_looping(false);
        expect_that!(&source.looping(), eq(false));
    }
}
