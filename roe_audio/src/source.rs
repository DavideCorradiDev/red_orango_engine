use super::{Error, Format};

pub use alto::DistanceModel;

pub trait Source {
    fn format(&self) -> Format;
    fn sample_rate(&self) -> u32;

    fn playing(&self) -> bool;
    fn play(&mut self) -> Result<(), Error>;
    fn pause(&mut self);
    fn stop(&mut self);

    fn replay(&mut self) -> Result<(), Error> {
        self.stop();
        self.play()
    }

    fn looping(&self) -> bool;
    fn set_looping(&mut self, value: bool);

    fn sample_length(&self) -> usize;
    fn sample_offset(&self) -> usize;
    fn set_sample_offset(&mut self, value: usize) -> Result<(), Error>;

    fn byte_length(&self) -> usize {
        self.sample_length() * self.format().total_bytes_per_sample() as usize
    }

    fn byte_offset(&self) -> usize {
        self.sample_offset() * self.format().total_bytes_per_sample() as usize
    }

    fn set_byte_offset(&mut self, value: usize) -> Result<(), Error> {
        let tbps = self.format().total_bytes_per_sample() as usize;
        assert!(
            value % tbps == 0,
            "Byte offset is within sample ({})",
            value
        );
        self.set_sample_offset(value / tbps)
    }

    fn time_length(&self) -> std::time::Duration {
        let sample_rate = self.sample_rate();
        assert!(sample_rate != 0);
        std::time::Duration::from_secs_f64(self.sample_length() as f64 / sample_rate as f64)
    }

    fn time_offset(&self) -> std::time::Duration {
        let sample_rate = self.sample_rate();
        assert!(sample_rate != 0);
        std::time::Duration::from_secs_f64(self.sample_offset() as f64 / sample_rate as f64)
    }

    fn set_time_offset(&mut self, value: std::time::Duration) -> Result<(), Error> {
        self.set_sample_offset((value.as_secs_f64() * self.sample_rate() as f64) as usize)
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

#[macro_export]
macro_rules! generate_source_tests {
    ($SourceGenerator:ty) => {
        // Properties tests.

        #[test]
        #[serial_test::serial]
        fn looping() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            expect_that!(&source.looping(), eq(false));

            source.set_looping(true);
            expect_that!(&source.looping(), eq(true));

            source.set_looping(false);
            expect_that!(&source.looping(), eq(false));
        }

        // Offset tests.

        #[test]
        #[serial_test::serial]
        fn set_sample_offset_while_paused() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            expect_that!(&source.sample_offset(), eq(0));
            source.set_sample_offset(24).unwrap();
            expect_that!(&source.sample_offset(), eq(24));
        }

        #[test]
        #[serial_test::serial]
        fn set_sample_offset_while_playing() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            expect_that!(&source.sample_offset(), eq(0));
            source.play().unwrap();
            expect_that!(&source.sample_offset(), not(geq(24)));
            source.set_sample_offset(24).unwrap();
            expect_that!(&source.sample_offset(), geq(24));
        }

        #[test]
        #[serial_test::serial]
        fn get_sample_offset_after_play() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            source.set_sample_offset(24).unwrap();
            expect_that!(&source.sample_offset(), eq(24));
            source.play().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
            expect_that!(&source.sample_offset(), gt(24));
        }

        #[test]
        #[serial_test::serial]
        fn get_sample_offset_after_pause() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            source.set_sample_offset(100).unwrap();
        }

        #[test]
        #[serial_test::serial]
        fn set_time_offset() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            expect_that!(&source.time_offset().as_secs_f64(), close_to(0., 1e-6));

            source
                .set_time_offset(std::time::Duration::from_secs_f64(0.3))
                .unwrap();
            expect_that!(&source.time_offset().as_secs_f64(), close_to(0.3, 1e-2));

            source
                .set_time_offset(std::time::Duration::from_secs_f64(0.))
                .unwrap();
            expect_that!(&source.time_offset().as_secs_f64(), close_to(0., 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "underflow when converting float to duration")]
        fn set_time_offset_negative() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            source
                .set_time_offset(std::time::Duration::from_secs_f64(-1.))
                .unwrap();
        }

        #[test]
        #[serial_test::serial]
        fn set_byte_offset() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            source.set_byte_offset(3).unwrap();
        }

        // Playback tests.

        #[test]
        #[serial_test::serial]
        fn play_at_initial_state() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            let pos0 = source.sample_offset();

            source.play().unwrap();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));
            expect_that!(&pos1, geq(pos0));
        }

        #[test]
        #[serial_test::serial]
        fn play_after_play() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
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

        #[test]
        #[serial_test::serial]
        fn stop_at_initial_state() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            source.set_sample_offset(24).unwrap();

            source.stop();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(false));
            expect_that!(&pos1, eq(0));
        }

        #[test]
        #[serial_test::serial]
        fn stop_after_play() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            let pos0 = source.sample_offset();

            source.play().unwrap();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));

            source.stop();
            let pos2 = source.sample_offset();
            expect_that!(&source.playing(), eq(false));

            expect_that!(&pos1, geq(pos0));
            expect_that!(&pos2, eq(0));
        }

        #[test]
        #[serial_test::serial]
        fn stop_after_pause() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            let pos0 = source.sample_offset();

            source.play().unwrap();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));

            source.pause();
            let pos2 = source.sample_offset();
            expect_that!(&source.playing(), eq(false));

            source.stop();
            let pos3 = source.sample_offset();
            expect_that!(&source.playing(), eq(false));

            expect_that!(&pos1, geq(pos0));
            expect_that!(&pos2, geq(pos1));
            expect_that!(&pos3, eq(0));
        }

        #[test]
        #[serial_test::serial]
        fn stop_after_stop() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            let pos0 = source.sample_offset();

            source.play().unwrap();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));

            source.stop();
            let pos2 = source.sample_offset();
            expect_that!(&source.playing(), eq(false));

            source.stop();
            let pos3 = source.sample_offset();
            expect_that!(&source.playing(), eq(false));

            expect_that!(&pos1, geq(pos0));
            expect_that!(&pos2, eq(0));
            expect_that!(&pos3, eq(0));
        }

        #[test]
        #[serial_test::serial]
        fn replay_at_initial_state() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            let pos0 = source.sample_offset();

            source.replay().unwrap();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));
            expect_that!(&pos1, geq(pos0));
        }

        #[test]
        #[serial_test::serial]
        fn replay_after_play() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            source.set_sample_offset(24).unwrap();
            let pos0 = source.sample_offset();

            source.play().unwrap();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));

            source.replay().unwrap();
            let pos2 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));

            expect_that!(&pos1, geq(pos0));
            expect_that!(&pos2, lt(pos1));
        }

        #[test]
        #[serial_test::serial]
        fn replay_after_pause() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            source.set_sample_offset(24).unwrap();
            let pos0 = source.sample_offset();

            source.play().unwrap();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));

            source.pause();
            let pos2 = source.sample_offset();
            expect_that!(&source.playing(), eq(false));

            source.replay().unwrap();
            let pos3 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));

            expect_that!(&pos1, geq(pos0));
            expect_that!(&pos2, geq(pos1));
            expect_that!(&pos3, lt(pos2));
        }

        #[test]
        #[serial_test::serial]
        fn replay_after_stop() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 64);
            let pos0 = source.sample_offset();

            source.play().unwrap();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));

            source.stop();
            let pos2 = source.sample_offset();
            expect_that!(&source.playing(), eq(false));

            source.replay().unwrap();
            let pos3 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));
            expect_that!(&pos1, geq(pos0));
            expect_that!(&pos2, eq(0));
            expect_that!(&pos3, geq(0));
        }

        #[test]
        #[serial_test::serial]
        fn play_looping() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 4000);
            source.set_looping(true);
            source.play().unwrap();
            expect_that!(&source.playing(), eq(true));
            std::thread::sleep(std::time::Duration::from_millis(50));
            expect_that!(&source.playing(), eq(true));
        }

        #[test]
        #[serial_test::serial]
        fn set_loping_while_playing() {
            let mut source = <$SourceGenerator>::create_with_buffer(Format::Stereo16, 64, 4000);
            source.play().unwrap();
            expect_that!(&source.playing(), eq(true));
            source.set_looping(true);
            std::thread::sleep(std::time::Duration::from_millis(50));
            expect_that!(&source.playing(), eq(true));
        }

        #[test]
        #[serial_test::serial]
        fn play_with_no_buffer_not_looping() {
            let mut source = <$SourceGenerator>::create_empty();
            source.play().unwrap();
            expect_that!(&source.playing(), eq(false));
        }

        #[test]
        #[serial_test::serial]
        fn play_with_no_buffer_looping() {
            let mut source = <$SourceGenerator>::create_empty();
            source.set_looping(true);
            source.play().unwrap();
            expect_that!(&source.playing(), eq(false));
        }
    };
}
