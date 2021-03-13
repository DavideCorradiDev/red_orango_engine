use roe_app::{
    application::Application,
    event::{keyboard, ControlFlow, DeviceId, EventHandler, EventLoop},
    window,
    window::{WindowBuilder, WindowId},
};

use roe_graphics::{
    CanvasWindow, CanvasWindowDescriptor, Instance, InstanceDescriptor, SampleCount,
};

use roe_examples::*;

struct ApplicationImplStartupData {
    stream: roe_audio::OutputStream,
    stream_handle: roe_audio::OutputStreamHandle,
}

struct ApplicationImpl {
    window: CanvasWindow,
    instance: Instance,
    stream: roe_audio::OutputStream,
    stream_handle: roe_audio::OutputStreamHandle,
}

impl ApplicationImpl {
    const SAMPLE_COUNT: SampleCount = 8;
}

impl EventHandler<ApplicationError, ApplicationEvent, ApplicationImplStartupData>
    for ApplicationImpl
{
    type Error = ApplicationError;
    type CustomEvent = ApplicationEvent;
    type StartupData = ApplicationImplStartupData;

    fn create_startup_data() -> Result<Self::StartupData, Self::Error> {
        let (stream, stream_handle) = roe_audio::OutputStream::try_default()?;
        Ok(Self::StartupData {
            stream,
            stream_handle,
        })
    }

    fn new(
        event_loop: &EventLoop<Self::CustomEvent>,
        startup_data: Self::StartupData,
    ) -> Result<Self, Self::Error> {
        let window = WindowBuilder::new()
            .with_inner_size(window::Size::Physical(window::PhysicalSize {
                width: 800,
                height: 800,
            }))
            .build(event_loop)?;
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
            stream: startup_data.stream,
            stream_handle: startup_data.stream_handle,
        })
    }
}

fn main() {
    const FIXED_FRAMERATE: u64 = 30;
    const VARIABLE_FRAMERATE_CAP: u64 = 60;
    Application::<ApplicationImpl, _, _, _>::new(FIXED_FRAMERATE, Some(VARIABLE_FRAMERATE_CAP))
        .run();
}
