use roe_app::{Application, ApplicationState, ControlFlow};

use roe_os as os;

use roe_audio::Source;

use roe_assets::{AudioBufferCache, AudioDecoderCache};

use std::{path::PathBuf, rc::Rc};

use roe_examples::*;

#[derive(Debug)]
struct ApplicationImpl {
    window: os::Window,
    static_source: roe_audio::StaticSource,
    streaming_source: roe_audio::StreamingSource,
}

impl ApplicationImpl {
    fn new(event_loop: &os::EventLoop<ApplicationEvent>) -> Result<Self, ApplicationError> {
        let window = os::WindowBuilder::new()
            .with_title("Sound Player")
            .with_inner_size(os::Size::Physical(os::PhysicalSize {
                width: 800,
                height: 600,
            }))
            .build(event_loop)?;
        let audio_device = roe_audio::Device::default()?;
        let audio_context = Rc::new(roe_audio::Context::default(&audio_device)?);

        let mut buffer_cache = AudioBufferCache::new(
            Rc::clone(&audio_context),
            PathBuf::from("roe_examples/data/audio"),
        );

        let mut decoder_cache = AudioDecoderCache::new(PathBuf::from("roe_examples/data/audio"));

        let audio_buffer = buffer_cache.get_or_load("stereo-16-44100.wav")?;
        let static_source = roe_audio::StaticSource::with_buffer(&audio_context, &audio_buffer)?;

        let audio_decoder = decoder_cache.remove_or_load("bach.ogg")?;
        let streaming_source = roe_audio::StreamingSource::with_decoder(
            &audio_context,
            audio_decoder,
            &roe_audio::StreamingSourceDescriptor::default(),
        )?;

        Ok(Self {
            window,
            static_source,
            streaming_source,
        })
    }
}

impl ApplicationState<ApplicationError, ApplicationEvent> for ApplicationImpl {
    fn on_key_pressed(
        &mut self,
        wid: os::WindowId,
        _device_id: os::DeviceId,
        _scan_code: os::ScanCode,
        key_code: Option<os::KeyCode>,
        _is_synthetic: bool,
        is_repeat: bool,
    ) -> Result<(), ApplicationError> {
        if !is_repeat && wid == self.window.id() {
            if let Some(key_code) = key_code {
                if key_code == os::KeyCode::Q {
                    self.static_source.play()?;
                }
                if key_code == os::KeyCode::W {
                    self.static_source.replay()?;
                }
                if key_code == os::KeyCode::E {
                    self.static_source.pause();
                }
                if key_code == os::KeyCode::R {
                    self.static_source.stop();
                }
                if key_code == os::KeyCode::T {
                    let cur_time = self.static_source.time_offset();
                    let time_step = std::time::Duration::from_secs_f64(0.1);
                    let new_time = if time_step > cur_time {
                        std::time::Duration::from_millis(0)
                    } else {
                        cur_time - time_step
                    };
                    self.static_source.set_time_offset(new_time)?;
                }
                if key_code == os::KeyCode::Y {
                    self.static_source.set_time_offset(
                        self.static_source.time_offset() + std::time::Duration::from_secs_f64(0.1),
                    )?;
                }

                if key_code == os::KeyCode::A {
                    self.streaming_source.play()?;
                }
                if key_code == os::KeyCode::S {
                    self.streaming_source.replay()?;
                }
                if key_code == os::KeyCode::D {
                    self.streaming_source.pause();
                }
                if key_code == os::KeyCode::F {
                    self.streaming_source.stop();
                }
                if key_code == os::KeyCode::G {
                    let cur_time = self.static_source.time_offset();
                    let time_step = std::time::Duration::from_secs_f64(0.1);
                    let new_time = if time_step > cur_time {
                        std::time::Duration::from_millis(0)
                    } else {
                        cur_time - time_step
                    };
                    self.streaming_source.set_time_offset(new_time)?;
                }
                if key_code == os::KeyCode::H {
                    self.streaming_source.set_time_offset(
                        self.streaming_source.time_offset()
                            + std::time::Duration::from_secs_f64(1.),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn on_fixed_update(&mut self, _: std::time::Duration) -> Result<(), ApplicationError> {
        self.streaming_source.update_buffers()?;
        Ok(())
    }
}

fn main() {
    const FIXED_FRAMERATE: u64 = 30;
    const VARIABLE_FRAMERATE_CAP: u64 = 60;
    Application::new(FIXED_FRAMERATE, Some(VARIABLE_FRAMERATE_CAP))
        .run(|event_queue| Ok(Box::new(ApplicationImpl::new(event_queue)?)));
}
