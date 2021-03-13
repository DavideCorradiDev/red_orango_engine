use roe_app::{
    application::Application,
    event::{keyboard, ControlFlow, DeviceId, EventHandler, EventLoop},
    window,
    window::{WindowBuilder, WindowId},
};

use roe_math::{
    conversion::convert,
    geometry2::{OrthographicProjection, Similarity, Translation, UnitComplex},
};

use roe_graphics::{
    AddressMode, Canvas, CanvasColorBufferFormat, CanvasColorBufferUsage, CanvasTexture,
    CanvasTextureColorBufferDescriptor, CanvasTextureDescriptor, CanvasWindow,
    CanvasWindowDescriptor, ColorF32, ColorF64, ColorOperations, CommandSequence, Instance,
    InstanceDescriptor, LoadOp, RenderPassOperations, SampleCount, Sampler, SamplerDescriptor,
    Size,
};

use roe_shape2::Renderer as Shape2Renderer;
use roe_sprite::{MeshTemplates as SpriteMeshTemplates, Renderer as SpriteRenderer};

use roe_examples::*;

#[derive(Debug)]
struct ApplicationImpl {
    window: CanvasWindow,
    instance: Instance,
}

impl ApplicationImpl {
    const SAMPLE_COUNT: SampleCount = 8;
}

impl EventHandler<ApplicationError, ApplicationEvent> for ApplicationImpl {
    type Error = ApplicationError;
    type CustomEvent = ApplicationEvent;

    fn new(event_loop: &EventLoop<Self::CustomEvent>) -> Result<Self, Self::Error> {
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
        Ok(Self { window, instance })
    }
}

fn main() {
    const FIXED_FRAMERATE: u64 = 30;
    const VARIABLE_FRAMERATE_CAP: u64 = 60;
    Application::<ApplicationImpl, _, _>::new(FIXED_FRAMERATE, Some(VARIABLE_FRAMERATE_CAP)).run();
}
