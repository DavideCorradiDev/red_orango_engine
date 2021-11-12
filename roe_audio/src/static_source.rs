use super::{AudioError, AudioFormat, Buffer, Context, DistanceModel, Source, SourceState};

use alto::Source as AltoSource;

use std::sync::Arc;

pub struct StaticSource {
    value: alto::StaticSource,
    audio_format: AudioFormat,
    sample_length: usize,
    sample_rate: u32,
    // Variable used to ensure consistency when retrieving the current sample offset when the source
    // is not playing.
    sample_offset_override: u64,
}

impl StaticSource {
    pub fn new(context: &Context) -> Result<Self, AudioError> {
        let static_source = context.value.new_static_source()?;
        Ok(Self {
            value: static_source,
            audio_format: AudioFormat::Mono8,
            sample_length: 0,
            sample_rate: 1,
            sample_offset_override: 0,
        })
    }
    pub fn with_buffer(context: &Context, buf: &Buffer) -> Result<Self, AudioError> {
        let mut static_source = Self::new(context)?;
        static_source.set_buffer(buf)?;
        Ok(static_source)
    }

    pub fn set_buffer(&mut self, buf: &Buffer) -> Result<(), AudioError> {
        self.value.set_buffer(Arc::clone(&buf.value))?;
        self.audio_format = buf.audio_format();
        self.sample_length = buf.sample_count();
        self.sample_rate = buf.sample_rate();
        self.sample_offset_override = 0;
        Ok(())
    }
}

impl Source for StaticSource {
    fn audio_format(&self) -> AudioFormat {
        self.audio_format
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn state(&self) -> SourceState {
        self.value.state()
    }

    fn play(&mut self) {
        if self.state() != SourceState::Playing {
            self.value.play();
        }
    }

    fn pause(&mut self) {
        if self.state() == SourceState::Playing {
            self.value.pause();
            self.sample_offset_override = self.sample_offset();
        }
    }

    fn stop(&mut self) {
        if self.state() == SourceState::Playing {
            self.value.stop();
            self.sample_offset_override = self.sample_offset();
        }
    }

    fn rewind(&mut self) {
        self.value.rewind();
        self.sample_offset_override = 0;
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

    fn sample_length(&self) -> usize {
        self.sample_length
    }

    fn sample_offset(&self) -> u64 {
        if self.value.state() == SourceState::Playing {
            self.value.sample_offset() as u64
        } else {
            self.sample_offset_override
        }
    }

    fn set_sample_offset(&mut self, value: u64) -> Result<(), AudioError> {
        self.value.set_sample_offset(value as alto::sys::ALint)?;
        self.sample_offset_override = std::cmp::min(value, self.sample_length() as u64);
        Ok(())
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
        super::{AudioFormat, Device, SourceState},
        *,
    };
    use galvanic_assert::{matchers::*, *};

    fn create_context() -> Context {
        let device = Device::default().unwrap();
        Context::default(&device).unwrap()
    }

    // TODO: test creation before setting buffer.
    // TODO: test creation with buffer set.
    // TODO: test setting buffer afterwards.
    // TODO: test individual properties with setters / getters.
    // TODO: test play / stop / pause etc (hard).
    // TODO: test looping (hard).

    #[test]
    #[serial_test::serial]
    fn creation() {
        let context = create_context();
        let source = StaticSource::new(&context).unwrap();
        expect_that!(&source.audio_format(), eq(AudioFormat::Mono8));
        expect_that!(&source.sample_rate(), eq(1));
        expect_that!(&source.state(), eq(SourceState::Initial));
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
        expect_that!(&source.sample_length(), eq(0));
        expect_that!(&source.sample_offset(), eq(0));
    }

    #[test]
    #[serial_test::serial]
    fn creation_with_buffer() {
        let context = create_context();
        let buf = Buffer::new(&context, &[0; 256], AudioFormat::Stereo16, 10).unwrap();
        let source = StaticSource::with_buffer(&context, &buf).unwrap();
        expect_that!(&source.audio_format(), eq(AudioFormat::Stereo16));
        expect_that!(&source.sample_rate(), eq(10));
        expect_that!(&source.state(), eq(SourceState::Initial));
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
        expect_that!(&source.sample_length(), eq(64));
        expect_that!(&source.sample_offset(), eq(0));
    }

    #[test]
    #[serial_test::serial]
    fn sample_offset() {
        let context = create_context();
        let buf = Buffer::new(&context, &[0; 256], AudioFormat::Stereo16, 10).unwrap();
        let mut source = StaticSource::with_buffer(&context, &buf).unwrap();
        expect_that!(&source.sample_length(), eq(64));
        expect_that!(&source.sample_offset(), eq(0));
        source.set_sample_offset(24).unwrap();
        expect_that!(&source.sample_offset(), eq(24));
        source.set_sample_offset(64).unwrap();
        expect_that!(&source.sample_offset(), eq(64));
        source.set_sample_offset(80).unwrap();
        expect_that!(&source.sample_offset(), eq(64));
    }

    #[test]
    #[serial_test::serial]
    fn sec_offset() {
        let context = create_context();
        let buf = Buffer::new(&context, &[0; 256], AudioFormat::Stereo16, 10).unwrap();
        let mut source = StaticSource::with_buffer(&context, &buf).unwrap();
        expect_that!(&source.sec_length(), close_to(6.4, 1e-6));
        expect_that!(&source.sec_offset(), close_to(0., 1e-6));
        source.set_sec_offset(3.1).unwrap();
        expect_that!(&source.sec_offset(), close_to(3.1, 1e-6));
        source.set_sec_offset(6.4).unwrap();
        expect_that!(&source.sec_offset(), close_to(6.4, 1e-6));
        source.set_sec_offset(8.).unwrap();
        expect_that!(&source.sec_offset(), close_to(6.4, 1e-6));
    }

    #[test]
    #[serial_test::serial]
    fn byte_offset() {
        let context = create_context();
        let buf = Buffer::new(&context, &[0; 256], AudioFormat::Stereo16, 10).unwrap();
        let mut source = StaticSource::with_buffer(&context, &buf).unwrap();
        expect_that!(&source.byte_length(), eq(256));
        expect_that!(&source.byte_offset(), eq(0));
        source.set_byte_offset(24).unwrap();
        expect_that!(&source.byte_offset(), eq(24));
        source.set_byte_offset(256).unwrap();
        expect_that!(&source.byte_offset(), eq(256));
        source.set_byte_offset(260).unwrap();
        expect_that!(&source.byte_offset(), eq(256));
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "Invalid byte offset (3)")]
    fn invalid_byte_offset() {
        let context = create_context();
        let buf = Buffer::new(&context, &[0; 256], AudioFormat::Stereo16, 10).unwrap();
        let mut source = StaticSource::with_buffer(&context, &buf).unwrap();
        source.set_byte_offset(3).unwrap();
    }
}
