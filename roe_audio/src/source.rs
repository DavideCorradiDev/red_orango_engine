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

    fn sample_length(&self) -> u64;
    fn sample_offset(&self) -> u64;
    fn set_sample_offset(&mut self, value: u64) -> Result<(), Error>;

    fn byte_length(&self) -> u64 {
        self.sample_length() * self.format().total_bytes_per_sample() as u64
    }

    fn byte_offset(&self) -> u64 {
        self.sample_offset() * self.format().total_bytes_per_sample() as u64
    }

    fn set_byte_offset(&mut self, value: u64) -> Result<(), Error> {
        let tbps = self.format().total_bytes_per_sample() as u64;
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
        self.set_sample_offset((value.as_secs_f64() * self.sample_rate() as f64) as u64)
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

    fn cone_inner_angle(&self) -> f32;
    fn set_cone_inner_angle(&mut self, value: f32);

    fn cone_outer_angle(&self) -> f32;
    fn set_cone_outer_angle(&mut self, value: f32);

    fn cone_outer_gain(&self) -> f32;
    fn set_cone_outer_gain(&mut self, value: f32);

    fn radius(&self) -> f32;
    fn set_radius(&mut self, value: f32);

    fn distance_model(&self) -> DistanceModel;
    fn set_distance_model(&mut self, value: DistanceModel);

    fn position<V: From<[f32; 3]>>(&self) -> V;
    fn set_position<V: Into<[f32; 3]>>(&mut self, value: V);

    fn velocity<V: From<[f32; 3]>>(&self) -> V;
    fn set_velocity<V: Into<[f32; 3]>>(&mut self, value: V);

    fn direction<V: From<[f32; 3]>>(&self) -> V;
    fn set_direction<V: Into<[f32; 3]>>(&mut self, value: V);
}

#[macro_export]
macro_rules! generate_source_tests {
    ($TestFixture:ty) => {
        // Creation tests.

        fn create_context() -> Context {
            let device = Device::default().unwrap();
            Context::default(&device).unwrap()
        }

        #[test]
        #[serial_test::serial]
        fn creation_empty() {
            let context = create_context();
            let source = <$TestFixture>::create_empty(&context);
            expect_that!(&source.playing(), eq(false));
            expect_that!(&source.format(), eq(Format::Mono8));
            expect_that!(&source.sample_rate(), eq(1));
            expect_that!(&source.sample_length(), eq(0));
            expect_that!(&source.sample_offset(), eq(0));
        }

        #[test]
        #[serial_test::serial]
        fn creation_non_empty() {
            let context = create_context();
            let source = <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 10);
            expect_that!(&source.playing(), eq(false));
            expect_that!(&source.format(), eq(Format::Stereo16));
            expect_that!(&source.sample_rate(), eq(10));
            expect_that!(&source.sample_length(), eq(64));
            expect_that!(&source.sample_offset(), eq(0));
        }

        // Set data.

        #[test]
        #[serial_test::serial]
        fn set_buffer() {
            let context = create_context();
            let mut source = <$TestFixture>::create_empty(&context);
            <$TestFixture>::set_data(&context, &mut source, Format::Stereo16, 64, 10);
            expect_that!(&source.playing(), eq(false));
            expect_that!(&source.format(), eq(Format::Stereo16));
            expect_that!(&source.sample_rate(), eq(10));
            expect_that!(&source.sample_length(), eq(64));
            expect_that!(&source.sample_offset(), eq(0));
        }

        #[test]
        #[serial_test::serial]
        fn set_buffer_while_playing() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 256, 100);
            source.set_looping(true);
            source.play().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
            expect_that!(&source.playing(), eq(true));
            expect_that!(&source.sample_offset(), gt(0));
            <$TestFixture>::set_data(&context, &mut source, Format::Stereo16, 256, 100);
            expect_that!(&source.playing(), eq(false));
            expect_that!(&source.sample_offset(), eq(0));
        }

        #[test]
        #[serial_test::serial]
        fn set_buffer_while_paused() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 256, 100);
            source.set_looping(true);
            source.play().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
            source.pause();
            expect_that!(&source.playing(), eq(false));
            expect_that!(&source.sample_offset(), gt(0));
            <$TestFixture>::set_data(&context, &mut source, Format::Stereo16, 256, 100);
            expect_that!(&source.playing(), eq(false));
            expect_that!(&source.sample_offset(), eq(0));
        }

        #[test]
        #[serial_test::serial]
        fn clear_buffer() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 256, 100);
            <$TestFixture>::clear_data(&mut source);
            expect_that!(&source.playing(), eq(false));
            expect_that!(&source.format(), eq(Format::Mono8));
            expect_that!(&source.sample_rate(), eq(1));
            expect_that!(&source.sample_length(), eq(0));
            expect_that!(&source.sample_offset(), eq(0));
        }

        #[test]
        #[serial_test::serial]
        fn clear_buffer_while_playing() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 256, 100);
            source.set_looping(true);
            source.play().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
            expect_that!(&source.playing(), eq(true));
            expect_that!(&source.sample_offset(), gt(0));
            <$TestFixture>::clear_data(&mut source);
            expect_that!(&source.playing(), eq(false));
            expect_that!(&source.sample_offset(), eq(0));
        }

        #[test]
        #[serial_test::serial]
        fn clear_buffer_while_paused() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 256, 100);
            source.set_looping(true);
            source.play().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
            source.pause();
            expect_that!(&source.playing(), eq(false));
            expect_that!(&source.sample_offset(), gt(0));
            <$TestFixture>::clear_data(&mut source);
            expect_that!(&source.playing(), eq(false));
            expect_that!(&source.sample_offset(), eq(0));
        }

        // Properties tests.

        #[test]
        #[serial_test::serial]
        fn looping() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.looping(), eq(false));

            source.set_looping(true);
            expect_that!(&source.looping(), eq(true));

            source.set_looping(false);
            expect_that!(&source.looping(), eq(false));
        }

        #[test]
        #[serial_test::serial]
        fn gain() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.gain(), close_to(1., 1e-6));
            source.set_gain(0.5);
            expect_that!(&source.gain(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "InvalidValue")]
        fn negative_gain() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_gain(-1.);
        }

        #[test]
        #[serial_test::serial]
        fn min_gain() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.min_gain(), close_to(0., 1e-6));
            source.set_min_gain(0.5);
            expect_that!(&source.min_gain(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "InvalidValue")]
        fn negative_min_gain() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_min_gain(-1.);
        }

        #[test]
        #[serial_test::serial]
        fn max_gain() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.max_gain(), close_to(1., 1e-6));
            source.set_max_gain(0.5);
            expect_that!(&source.max_gain(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "InvalidValue")]
        fn negative_max_gain() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_max_gain(-1.);
        }

        #[test]
        #[serial_test::serial]
        fn reference_distance() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.reference_distance(), close_to(1., 1e-6));
            source.set_reference_distance(0.5);
            expect_that!(&source.reference_distance(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "InvalidValue")]
        fn negative_reference_distance() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_reference_distance(-1.);
        }

        #[test]
        #[serial_test::serial]
        fn rolloff_factor() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.rolloff_factor(), close_to(1., 1e-6));
            source.set_rolloff_factor(0.5);
            expect_that!(&source.rolloff_factor(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "InvalidValue")]
        fn negative_rolloff_factor() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_rolloff_factor(-1.);
        }

        #[test]
        #[serial_test::serial]
        fn max_distance() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.max_distance(), close_to(34028235e31, 1e-6));
            source.set_max_distance(0.5);
            expect_that!(&source.max_distance(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "InvalidValue")]
        fn negative_max_distance() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_max_distance(-1.);
        }

        #[test]
        #[serial_test::serial]
        fn pitch() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.pitch(), close_to(1., 1e-6));
            source.set_pitch(0.5);
            expect_that!(&source.pitch(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "InvalidValue")]
        fn negative_pitch() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_pitch(-1.);
        }

        #[test]
        #[serial_test::serial]
        fn cone_inner_angle() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(
                &source.cone_inner_angle(),
                close_to(2. * std::f32::consts::PI, 1e-6)
            );
            source.set_cone_inner_angle(0.5);
            expect_that!(&source.cone_inner_angle(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "InvalidValue")]
        fn negative_cone_inner_angle() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_cone_inner_angle(-1.);
        }

        #[test]
        #[serial_test::serial]
        fn cone_outer_angle() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(
                &source.cone_outer_angle(),
                close_to(2. * std::f32::consts::PI, 1e-6)
            );
            source.set_cone_outer_angle(0.5);
            expect_that!(&source.cone_outer_angle(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "InvalidValue")]
        fn negative_cone_outer_angle() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_cone_outer_angle(-1.);
        }

        #[test]
        #[serial_test::serial]
        fn cone_outer_gain() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.cone_outer_gain(), close_to(0., 1e-6));
            source.set_cone_outer_gain(0.5);
            expect_that!(&source.cone_outer_gain(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "InvalidValue")]
        fn negative_cone_outer_gain() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_cone_outer_gain(-1.);
        }

        #[test]
        #[serial_test::serial]
        fn radius() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.radius(), close_to(0., 1e-6));
            source.set_radius(0.5);
            expect_that!(&source.radius(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        #[should_panic(expected = "InvalidValue")]
        fn negative_radius() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_radius(-1.);
        }

        #[test]
        #[serial_test::serial]
        fn distance_model() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.distance_model(), eq(DistanceModel::InverseClamped));
            source.set_distance_model(DistanceModel::Exponent);
            expect_that!(&source.distance_model(), eq(DistanceModel::Exponent));
        }

        #[test]
        #[serial_test::serial]
        fn position() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.position(), eq([0., 0., 0.]));
            source.set_position([1., 2., 3.]);
            expect_that!(&source.position(), eq([1., 2., 3.]));
        }

        #[test]
        #[serial_test::serial]
        fn velocity() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.velocity(), eq([0., 0., 0.]));
            source.set_velocity([1., 2., 3.]);
            expect_that!(&source.velocity(), eq([1., 2., 3.]));
        }

        #[test]
        #[serial_test::serial]
        fn direction() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.direction(), eq([0., 0., 0.]));
            source.set_direction([1., 2., 3.]);
            expect_that!(&source.direction(), eq([1., 2., 3.]));
        }

        // Offset tests.

        #[test]
        #[serial_test::serial]
        fn length_mono8() {
            let context = create_context();
            let source = <$TestFixture>::create_with_data(&context, Format::Mono8, 64, 64);
            expect_that!(&source.sample_length(), eq(64));
            expect_that!(&source.byte_length(), eq(64));
            expect_that!(&source.time_length().as_secs_f32(), close_to(1., 1e-6));
        }

        #[test]
        #[serial_test::serial]
        fn length_mono16() {
            let context = create_context();
            let source = <$TestFixture>::create_with_data(&context, Format::Mono16, 64, 64);
            expect_that!(&source.sample_length(), eq(64));
            expect_that!(&source.byte_length(), eq(128));
            expect_that!(&source.time_length().as_secs_f32(), close_to(1., 1e-6));
        }

        #[test]
        #[serial_test::serial]
        fn length_stereo8() {
            let context = create_context();
            let source = <$TestFixture>::create_with_data(&context, Format::Stereo8, 64, 64);
            expect_that!(&source.sample_length(), eq(64));
            expect_that!(&source.byte_length(), eq(128));
            expect_that!(&source.time_length().as_secs_f32(), close_to(1., 1e-6));
        }

        #[test]
        #[serial_test::serial]
        fn length_stereo16() {
            let context = create_context();
            let source = <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.sample_length(), eq(64));
            expect_that!(&source.byte_length(), eq(256));
            expect_that!(&source.time_length().as_secs_f32(), close_to(1., 1e-6));
        }

        #[test]
        #[serial_test::serial]
        fn length_double_sample_rate() {
            let context = create_context();
            let source = <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 128);
            expect_that!(&source.sample_length(), eq(64));
            expect_that!(&source.byte_length(), eq(256));
            expect_that!(&source.time_length().as_secs_f32(), close_to(0.5, 1e-6));
        }

        #[test]
        #[serial_test::serial]
        fn set_sample_offset_while_paused() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.sample_offset(), eq(0));
            source.set_sample_offset(24).unwrap();
            expect_that!(&source.sample_offset(), eq(24));
        }

        #[test]
        #[serial_test::serial]
        fn set_sample_offset_while_playing() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            expect_that!(&source.sample_offset(), eq(0));
            source.play().unwrap();
            expect_that!(&source.sample_offset(), not(geq(24)));
            source.set_sample_offset(24).unwrap();
            expect_that!(&source.sample_offset(), geq(24));
        }

        #[test]
        #[serial_test::serial]
        fn get_sample_offset_after_play() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_sample_offset(24).unwrap();
            expect_that!(&source.sample_offset(), eq(24));
            source.play().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
            expect_that!(&source.sample_offset(), gt(24));
        }

        #[test]
        #[serial_test::serial]
        fn get_sample_offset_after_pause() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_sample_offset(100).unwrap();
        }

        #[test]
        #[serial_test::serial]
        fn set_time_offset() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source
                .set_time_offset(std::time::Duration::from_secs_f64(-1.))
                .unwrap();
        }

        #[test]
        #[serial_test::serial]
        fn set_byte_offset() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_byte_offset(3).unwrap();
        }

        // Playback tests.

        #[test]
        #[serial_test::serial]
        fn play_at_initial_state() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            let pos0 = source.sample_offset();

            source.play().unwrap();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));
            expect_that!(&pos1, geq(pos0));
        }

        #[test]
        #[serial_test::serial]
        fn play_after_play() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            source.set_sample_offset(24).unwrap();

            source.stop();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(false));
            expect_that!(&pos1, eq(0));
        }

        #[test]
        #[serial_test::serial]
        fn stop_after_play() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
            let pos0 = source.sample_offset();

            source.replay().unwrap();
            let pos1 = source.sample_offset();
            expect_that!(&source.playing(), eq(true));
            expect_that!(&pos1, geq(pos0));
        }

        #[test]
        #[serial_test::serial]
        fn replay_after_play() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 64);
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
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 4000);
            source.set_looping(true);
            source.play().unwrap();
            expect_that!(&source.playing(), eq(true));
            std::thread::sleep(std::time::Duration::from_millis(50));
            expect_that!(&source.playing(), eq(true));
        }

        #[test]
        #[serial_test::serial]
        fn set_loping_while_playing() {
            let context = create_context();
            let mut source =
                <$TestFixture>::create_with_data(&context, Format::Stereo16, 64, 4000);
            source.play().unwrap();
            expect_that!(&source.playing(), eq(true));
            source.set_looping(true);
            std::thread::sleep(std::time::Duration::from_millis(50));
            expect_that!(&source.playing(), eq(true));
        }

        #[test]
        #[serial_test::serial]
        fn play_with_no_buffer_not_looping() {
            let context = create_context();
            let mut source = <$TestFixture>::create_empty(&context);
            source.play().unwrap();
            expect_that!(&source.playing(), eq(false));
        }

        #[test]
        #[serial_test::serial]
        fn play_with_no_buffer_looping() {
            let context = create_context();
            let mut source = <$TestFixture>::create_empty(&context);
            source.set_looping(true);
            source.play().unwrap();
            expect_that!(&source.playing(), eq(false));
        }
    };
}
