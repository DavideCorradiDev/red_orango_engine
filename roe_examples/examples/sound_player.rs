use roe_app::{
    application::Application,
    event::{EventHandler, EventLoop},
    window::{PhysicalSize, Size, Window, WindowBuilder},
};

use roe_examples::*;

#[derive(Debug)]
struct ApplicationImpl {
    _window: Window,
    _audio_instance: roe_audio::Instance,
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
        // TODO: replace unwrap call.
        let audio_instance = roe_audio::Instance::new().unwrap();
        Ok(Self {
            _window: window,
            _audio_instance: audio_instance,
        })
    }
}

fn main() {
    const FIXED_FRAMERATE: u64 = 30;
    const VARIABLE_FRAMERATE_CAP: u64 = 60;
    Application::<ApplicationImpl, _, _>::new(FIXED_FRAMERATE, Some(VARIABLE_FRAMERATE_CAP)).run();
}
