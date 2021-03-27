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
        // TODO: replace unwrap
        let audio_buffer = roe_audio::Buffer::from_decoder(
            &audio_context,
            &mut roe_audio::WavDecoder::new(std::io::BufReader::new(
                std::fs::File::open("roe_examples/data/audio/stereo-16-44100.wav").unwrap(),
            ))
            .unwrap(),
        )
        .unwrap();
        let mut static_source = roe_audio::StaticSource::new(&audio_context)?;
        static_source.set_buffer(&audio_buffer)?;

        Ok(Self {
            window,
            audio_device,
            audio_context,
            static_source,
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
                if key_code == keyboard::KeyCode::Key1 {
                    self.static_source.play();
                }
            }
        }
        Ok(ControlFlow::Continue)
    }
}

fn main() {
    const FIXED_FRAMERATE: u64 = 30;
    const VARIABLE_FRAMERATE_CAP: u64 = 60;
    Application::<ApplicationImpl, _, _>::new(FIXED_FRAMERATE, Some(VARIABLE_FRAMERATE_CAP)).run();
}
