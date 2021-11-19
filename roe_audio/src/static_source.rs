use super::{Buffer, Context, DistanceModel, Error, Format, Source};

use alto::Source as AltoSource;

use std::sync::Arc;

pub struct StaticSource {
    value: alto::StaticSource,
    paused_sample_offset: u64,
}

impl StaticSource {
    pub fn new(context: &Context) -> Result<Self, Error> {
        let static_source = context.value.new_static_source()?;
        Ok(Self {
            value: static_source,
            paused_sample_offset: 0,
        })
    }
    pub fn with_buffer(context: &Context, buf: &Buffer) -> Result<Self, Error> {
        let mut static_source = Self::new(context)?;
        static_source.set_buffer(buf)?;
        Ok(static_source)
    }

    pub fn set_buffer(&mut self, buf: &Buffer) -> Result<(), Error> {
        self.stop();
        self.value.set_buffer(Arc::clone(&buf.value))?;
        self.paused_sample_offset = 0;
        Ok(())
    }

    pub fn clear_buffer(&mut self) {
        self.stop();
        self.value.clear_buffer();
        self.paused_sample_offset = 0;
    }

    fn normalize_sample_offset(&self, value: u64) -> u64 {
        let sample_length = self.sample_length();
        if sample_length == 0 {
            0
        } else if self.looping() {
            value % sample_length
        } else if value >= sample_length {
            0
        } else {
            value
        }
    }
}

impl Source for StaticSource {
    fn format(&self) -> Format {
        match self.value.buffer() {
            Some(b) => {
                let bytes_per_sample = b.bits() / 8;
                Format::new(b.channels() as u32, bytes_per_sample as u32)
            }
            None => Format::Mono8,
        }
    }

    fn sample_rate(&self) -> u32 {
        match self.value.buffer() {
            Some(b) => b.frequency() as u32,
            None => 1,
        }
    }

    fn playing(&self) -> bool {
        self.value.state() == alto::SourceState::Playing
    }

    fn play(&mut self) -> Result<(), Error> {
        if !self.playing() && self.sample_length() > 0 {
            self.value
                .set_sample_offset(self.paused_sample_offset as i32)?;
            self.paused_sample_offset = 0;
            self.value.play();
        }
        Ok(())
    }

    fn pause(&mut self) {
        if self.playing() {
            self.value.pause();
            self.paused_sample_offset = self.value.sample_offset() as u64;
            self.value.stop();
        }
    }

    fn stop(&mut self) {
        self.value.stop();
        self.paused_sample_offset = 0;
    }

    fn looping(&self) -> bool {
        self.value.looping()
    }

    fn set_looping(&mut self, value: bool) {
        self.value.set_looping(value)
    }

    fn byte_length(&self) -> u64 {
        match self.value.buffer() {
            Some(b) => b.size() as u64,
            None => 0,
        }
    }

    fn sample_length(&self) -> u64 {
        let byte_length = self.byte_length();
        let tbps = self.format().total_bytes_per_sample() as u64;
        assert!(byte_length % tbps == 0);
        byte_length / tbps
    }

    fn sample_offset(&self) -> u64 {
        if self.playing() {
            self.value.sample_offset() as u64
        } else {
            self.paused_sample_offset
        }
    }

    fn set_sample_offset(&mut self, value: u64) -> Result<(), Error> {
        let sample_offset = self.normalize_sample_offset(value);
        if self.playing() {
            self.value.stop();
            self.value
                .set_sample_offset(sample_offset as alto::sys::ALint)?;
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

impl std::fmt::Debug for StaticSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StaticSource {{ }}")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{generate_source_tests, Device, Format},
        *,
    };
    use galvanic_assert::{matchers::*, *};

    struct TestFixture {}

    impl TestFixture {
        fn create_empty(context: &Context) -> StaticSource {
            StaticSource::new(context).unwrap()
        }

        fn create_with_data(
            context: &Context,
            format: Format,
            sample_count: usize,
            sample_rate: u32,
        ) -> StaticSource {
            let buf = Buffer::new(
                context,
                vec![0; sample_count * format.total_bytes_per_sample() as usize].as_ref(),
                format,
                sample_rate,
            )
            .unwrap();
            StaticSource::with_buffer(&context, &buf).unwrap()
        }

        fn clear_data(source: &mut StaticSource) {
            source.clear_buffer();
        }

        fn set_data(
            context: &Context,
            source: &mut StaticSource,
            format: Format,
            sample_count: usize,
            sample_rate: u32,
        ) {
            let buf = Buffer::new(
                context,
                vec![0; sample_count * format.total_bytes_per_sample() as usize].as_ref(),
                format,
                sample_rate,
            )
            .unwrap();
            source.set_buffer(&buf).unwrap();
        }
    }

    generate_source_tests!(TestFixture);
}
