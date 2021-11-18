use roe_app::{
    application::Application,
    event::{keyboard, ControlFlow, DeviceId, EventHandler, EventLoop},
    window::{PhysicalSize, Size, Window, WindowBuilder, WindowId},
};

use roe_audio::Source;

use roe_examples::*;

#[derive(Debug)]
struct ApplicationImpl {
    window: Window,
    audio_device: roe_audio::Device,
    audio_context: roe_audio::Context,
    static_source: roe_audio::StaticSource,
    streaming_source: roe_audio::StreamingSource,
}

impl EventHandler<ApplicationError, ()> for ApplicationImpl {
    type Error = ApplicationError;
    type CustomEvent = ();

    fn new(event_loop: &EventLoop<Self::CustomEvent>) -> Result<Self, Self::Error> {
        let window = WindowBuilder::new()
            .with_title("Sound Player")
            .with_inner_size(Size::Physical(PhysicalSize {
                width: 800,
                height: 600,
            }))
            .build(event_loop)?;
        let audio_device = roe_audio::Device::default()?;
        let audio_context = roe_audio::Context::default(&audio_device)?;

        let audio_buffer = roe_audio::Buffer::from_decoder(
            &audio_context,
            &mut roe_audio::WavDecoder::new(std::io::BufReader::new(std::fs::File::open(
                "roe_examples/data/audio/stereo-16-44100.wav",
            )?))?,
        )?;
        let static_source = roe_audio::StaticSource::with_buffer(&audio_context, &audio_buffer)?;

        let streaming_source = roe_audio::StreamingSource::with_decoder(
            &audio_context,
            Box::new(roe_audio::OggDecoder::new(std::io::BufReader::new(
                std::fs::File::open("roe_examples/data/audio/bach.ogg")?,
            ))?),
            &roe_audio::StreamingSourceDescriptor::default(),
        )?;

        Ok(Self {
            window,
            audio_device,
            audio_context,
            static_source,
            streaming_source,
        })
    }

    fn on_key_pressed(
        &mut self,
        wid: WindowId,
        _device_id: DeviceId,
        _scan_code: keyboard::ScanCode,
        key_code: Option<keyboard::KeyCode>,
        _is_synthetic: bool,
        is_repeat: bool,
    ) -> Result<ControlFlow, Self::Error> {
        if !is_repeat && wid == self.window.id() {
            if let Some(key_code) = key_code {
                if key_code == keyboard::KeyCode::Q {
                    self.static_source.play()?;
                }
                if key_code == keyboard::KeyCode::W {
                    self.static_source.replay()?;
                }
                if key_code == keyboard::KeyCode::E {
                    self.static_source.pause();
                }
                if key_code == keyboard::KeyCode::R {
                    self.static_source.stop();
                }
                if key_code == keyboard::KeyCode::T {
                    let cur_time = self.static_source.time_offset();
                    let time_step = std::time::Duration::from_secs_f64(0.1);
                    let new_time = if time_step > cur_time {
                        std::time::Duration::from_millis(0)
                    }
                    else {
                        cur_time - time_step
                    };
                    self.static_source.set_time_offset(new_time)?;
                }
                if key_code == keyboard::KeyCode::Y {
                    self.static_source.set_time_offset(
                        self.static_source.time_offset() + std::time::Duration::from_secs_f64(0.1),
                    )?;
                }

                if key_code == keyboard::KeyCode::A {
                    self.streaming_source.play()?;
                }
                if key_code == keyboard::KeyCode::S {
                    self.streaming_source.replay()?;
                }
                if key_code == keyboard::KeyCode::D {
                    self.streaming_source.pause();
                }
                if key_code == keyboard::KeyCode::F {
                    self.streaming_source.stop();
                }
                if key_code == keyboard::KeyCode::G {
                    let cur_time = self.static_source.time_offset();
                    let time_step = std::time::Duration::from_secs_f64(0.1);
                    let new_time = if time_step > cur_time {
                        std::time::Duration::from_millis(0)
                    }
                    else {
                        cur_time - time_step
                    };
                    self.streaming_source.set_time_offset(new_time)?;
                }
                if key_code == keyboard::KeyCode::H {
                    self.streaming_source.set_time_offset(
                        self.streaming_source.time_offset() + std::time::Duration::from_secs_f64(1.),
                    )?;
                }
            }
        }
        Ok(ControlFlow::Continue)
    }

    fn on_fixed_update(&mut self, _: std::time::Duration) -> Result<ControlFlow, Self::Error> {
        self.streaming_source.update_buffers()?;
        Ok(ControlFlow::Continue)
    }
}

fn main() {
    const FIXED_FRAMERATE: u64 = 30;
    const VARIABLE_FRAMERATE_CAP: u64 = 60;
    Application::<ApplicationImpl, _, _>::new(FIXED_FRAMERATE, Some(VARIABLE_FRAMERATE_CAP)).run();
}
