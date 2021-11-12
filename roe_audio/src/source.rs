use super::{AudioError, AudioFormat};

pub use alto::{DistanceModel};

pub trait Source {
    fn audio_format(&self) -> AudioFormat;
    fn sample_rate(&self) -> u32;

    // TODO: add looping / set_looping to this trait.
    fn playing(&self) -> bool;
    fn play(&mut self) -> Result<(), AudioError>;
    fn pause(&mut self);
    fn stop(&mut self);

    fn replay(&mut self) -> Result<(), AudioError>
    {
        self.stop();
        self.play()
    }

    fn looping(&self) -> bool;
    fn set_looping(&mut self, value: bool);

    fn sample_length(&self) -> usize;
    fn sample_offset(&self) -> u64;
    fn set_sample_offset(&mut self, value: u64) -> Result<(), AudioError>;

    fn byte_length(&self) -> usize {
        self.sample_length() * self.audio_format().total_bytes_per_sample() as usize
    }

    // TODO: change to usize.
    fn byte_offset(&self) -> u64 {
        self.sample_offset() * self.audio_format().total_bytes_per_sample() as u64
    }

    fn set_byte_offset(&mut self, value: u64) -> Result<(), AudioError> {
        let tbps = self.audio_format().total_bytes_per_sample() as u64;
        assert!(value % tbps == 0, "Invalid byte offset ({})", value);
        self.set_sample_offset(value / tbps)
    }

    // TODO: use an appropriate time datatype.
    fn sec_length(&self) -> f32
    {
        let sample_rate = self.sample_rate();
        assert!(sample_rate != 0);
        self.sample_length() as f32 / sample_rate as f32
    }

    fn sec_offset(&self) -> f32
    {
        let sample_rate = self.sample_rate();
        assert!(sample_rate != 0);
        self.sample_offset() as f32 / sample_rate as f32
    }

    fn set_sec_offset(&mut self, value: f32) -> Result<(), AudioError>
    {
        self.set_sample_offset((value * self.sample_rate() as f32) as u64)
    }

    fn gain(&self) -> f32;
    fn set_gain(&mut self, value: f32);

    fn min_gain(&self) -> f32;
    fn set_min_gain(&mut self, value: f32);

    fn max_gain(&self) -> f32;
    fn set_max_gain(&mut self, value: f32);

    fn reference_distance(&self) -> f32;
    fn set_reference_distance(&mut self, value: f32);

    fn rolloff_factor(&self) -> f32;
    fn set_rolloff_factor(&mut self, value: f32);

    fn max_distance(&self) -> f32;
    fn set_max_distance(&mut self, value: f32);

    fn pitch(&self) -> f32;
    fn set_pitch(&mut self, value: f32);

    fn position<V: From<[f32; 3]>>(&self) -> V;
    fn set_position<V: Into<[f32; 3]>>(&mut self, value: V);

    fn velocity<V: From<[f32; 3]>>(&self) -> V;
    fn set_velocity<V: Into<[f32; 3]>>(&mut self, value: V);

    fn direction<V: From<[f32; 3]>>(&self) -> V;
    fn set_direction<V: Into<[f32; 3]>>(&mut self, value: V);

    fn cone_inner_angle(&self) -> f32;
    fn set_cone_inner_angle(&mut self, value: f32);

    fn cone_outer_angle(&self) -> f32;
    fn set_cone_outer_angle(&mut self, value: f32);

    fn cone_outer_gain(&self) -> f32;
    fn set_cone_outer_gain(&mut self, value: f32);

    fn distance_model(&self) -> DistanceModel;
    fn set_distance_model(&mut self, value: DistanceModel);

    fn radius(&self) -> f32;
    fn set_radius(&self, value: f32);
}
