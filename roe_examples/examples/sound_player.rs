use roe_app::{
    application::Application,
    event::{keyboard, ControlFlow, DeviceId, EventHandler, EventLoop},
    window::{PhysicalSize, Size, Window, WindowBuilder, WindowId},
};

use roe_examples::*;

#[derive(Debug)]
struct ApplicationImpl {
    window: Window,
    audio_device: roe_audio::Device,
    audio_context: roe_audio::Context,
    audio_mixer: roe_audio::Mixer,
    sound: roe_audio::Sound,
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
        let audio_mixer = roe_audio::Mixer::new(&audio_context)?;
        // TODO: replace unwrap
        let sound = roe_audio::Sound::from_decoder(
            &mut roe_audio::WavDecoder::new(std::io::BufReader::new(
                std::fs::File::open("roe_examples/data/audio/stereo-16-44100.wav").unwrap(),
            ))
            .unwrap(),
        )
        .unwrap();

        Ok(Self {
            window,
            audio_device,
            audio_context,
            audio_mixer,
            sound,
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
                    self.audio_mixer.play(&self.audio_context, &self.sound)?;
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
