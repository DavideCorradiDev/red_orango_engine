use roe_app::{
    application::Application,
    event::{keyboard, ControlFlow, DeviceId, EventHandler, EventLoop},
    window,
    window::{WindowBuilder, WindowBuilderExt, WindowId},
};

use roe_graphics::{
    CanvasWindow, CanvasWindowDescriptor, Instance, InstanceDescriptor, SampleCount,
};

use roe_audio::Source;

use roe_examples::*;

struct ApplicationImpl {
    window: CanvasWindow,
    instance: Instance,
    stream: roe_audio::OutputStream,
    stream_handle: roe_audio::OutputStreamHandle,
}

impl ApplicationImpl {
    const SAMPLE_COUNT: SampleCount = 8;
}

#[cfg(target_os = "windows")]
fn fix_window_builder(window_builder: WindowBuilder) -> WindowBuilder {
    window_builder.with_drag_and_drop(false)
}

#[cfg(not(target_os = "windows"))]
fn fix_window_builder(window_builder: WindowBuilder) -> WindowBuilder {
    window_builder
}

impl EventHandler<ApplicationError, ApplicationEvent> for ApplicationImpl {
    type Error = ApplicationError;
    type CustomEvent = ApplicationEvent;

    fn new(event_loop: &EventLoop<Self::CustomEvent>) -> Result<Self, Self::Error> {
        let (stream, stream_handle) = roe_audio::OutputStream::try_default()?;
        let mut window_builder =
            WindowBuilder::new().with_inner_size(window::Size::Physical(window::PhysicalSize {
                width: 800,
                height: 800,
            }));
        window_builder = fix_window_builder(window_builder);
        let window = window_builder.build(event_loop)?;
        let (window, instance) = unsafe {
            let (instance, surface) = Instance::new_with_compatible_window(
                &InstanceDescriptor::high_performance(),
                &window,
            )?;
            let window = CanvasWindow::from_window_and_surface(
                &instance,
                window,
                surface,
                &CanvasWindowDescriptor {
                    sample_count: Self::SAMPLE_COUNT,
                    ..CanvasWindowDescriptor::default()
                },
            );
            (window, instance)
        };
        Ok(Self {
            window,
            instance,
            stream,
            stream_handle,
        })
    }

    fn on_key_released(
        &mut self,
        wid: WindowId,
        _device_id: DeviceId,
        _scan_code: keyboard::ScanCode,
        key_code: Option<keyboard::KeyCode>,
        _is_synthetic: bool,
    ) -> Result<ControlFlow, Self::Error> {
        if wid == self.window.id() {
            if let Some(key) = key_code {
                if key == keyboard::KeyCode::Key1 {
                    // TODO: replace unwrap().
                    let wav_file =
                        std::fs::File::open("roe_examples/data/audio/stereo-8-44100.wav").unwrap();
                    let wav_audio =
                        roe_audio::Decoder::new(std::io::BufReader::new(wav_file)).unwrap();
                    self.stream_handle
                        .play_raw(wav_audio.convert_samples())
                        .unwrap();
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
