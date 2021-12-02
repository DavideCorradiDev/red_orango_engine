use roe_app::{
    application::Application,
    event::{ControlFlow, EventHandler, EventLoop},
    window,
    window::{WindowBuilder, WindowId},
};

use roe_math::{
    conversion::convert,
    geometry2::{OrthographicProjection, Projective, Translation},
};

use roe_graphics::{
    Canvas, CanvasWindow, CanvasWindowDescriptor, ColorF32, CommandSequence, Instance,
    InstanceDescriptor, RenderPassOperations, SampleCount,
};

use roe_text::Renderer as TextRenderer;

use roe_assets::FontCache;

use std::{path::PathBuf, rc::Rc};

use roe_examples::*;

#[derive(Debug)]
struct ApplicationImpl {
    window: CanvasWindow,
    instance: Rc<Instance>,
    projection_transform: Projective<f32>,
    pipeline: roe_text::RenderPipeline,
    font_cache: FontCache,
}

impl ApplicationImpl {
    const SAMPLE_COUNT: SampleCount = 4;
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
            (window, Rc::new(instance))
        };

        let window_size = window.inner_size();

        let projection_transform = OrthographicProjection::new(
            0.,
            window_size.width as f32,
            window_size.height as f32,
            0.,
        )
        .to_projective();

        let pipeline = roe_text::RenderPipeline::new(
            &instance,
            &roe_text::RenderPipelineDescriptor {
                sample_count: Self::SAMPLE_COUNT,
                ..roe_text::RenderPipelineDescriptor::default()
            },
        );

        let mut font_cache = FontCache::new(
            Rc::clone(&instance),
            Rc::new(roe_text::FontLibrary::new()?),
            PathBuf::from("roe_examples/data/fonts"),
            roe_text::character_set::english(),
        );
        font_cache.load("Roboto-Regular.ttf", 0, 11.)?;

        Ok(Self {
            window,
            instance,
            projection_transform,
            pipeline,
            font_cache,
        })
    }

    fn on_resized(
        &mut self,
        wid: WindowId,
        size: window::PhysicalSize<u32>,
    ) -> Result<ControlFlow, Self::Error> {
        if wid == self.window.id() {
            self.window.update_buffer(&self.instance);
            self.projection_transform = OrthographicProjection::new(
                0.,
                1f32.max(size.width as f32),
                1f32.max(size.height as f32),
                0.,
            )
            .to_projective();
        }
        Ok(ControlFlow::Continue)
    }

    fn on_variable_update(&mut self, _dt: std::time::Duration) -> Result<ControlFlow, Self::Error> {
        if let Some(frame) = self.window.current_frame()? {
            let mut cmd_sequence = CommandSequence::new(&self.instance);

            {
                let mut rpass = cmd_sequence.begin_render_pass(
                    &frame,
                    &self.pipeline.render_pass_requirements(),
                    &RenderPassOperations::default(),
                );
                {
                    rpass.draw_text(
                        &self.pipeline,
                        self.font_cache.get("Roboto-Regular.ttf", 0, 11.).unwrap(),
                        "Lorem ipsum dolor sit amet",
                        &convert(self.projection_transform * Translation::new(100., 100.)),
                        &ColorF32::BLUE,
                    );
                }
                {
                    rpass.draw_text(
                        &self.pipeline,
                        self.font_cache.get("Roboto-Regular.ttf", 0, 11.).unwrap(),
                        "Hello world!",
                        &convert(self.projection_transform * Translation::new(300., 300.)),
                        &ColorF32::RED,
                    );
                }
            }

            cmd_sequence.submit(&self.instance);
            frame.present();
        }
        Ok(ControlFlow::Continue)
    }
}

fn main() {
    const FIXED_FRAMERATE: u64 = 30;
    const VARIABLE_FRAMERATE_CAP: u64 = 60;
    Application::<ApplicationImpl, _, _>::new(FIXED_FRAMERATE, Some(VARIABLE_FRAMERATE_CAP)).run();
}
