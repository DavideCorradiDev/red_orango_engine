use roe_app::{Application, ApplicationState};

use roe_os as os;

use roe_math::{HomogeneousMatrix2, Vector2};

use roe_graphics::{
    Canvas, CanvasWindow, CanvasWindowDescriptor, ColorF32, CommandSequence, Instance,
    InstanceDescriptor, RenderPassOperations, SampleCount,
};

use roe_text::{character_set, Face, Font, FontLibrary, Renderer as TextRenderer};

use std::rc::Rc;

use roe_examples::*;

#[derive(Debug)]
struct ApplicationImpl {
    window: CanvasWindow,
    instance: Rc<Instance>,
    projection_transform: HomogeneousMatrix2<f32>,
    pipeline: roe_text::RenderPipeline,
    font: Font,
}

impl ApplicationImpl {
    const SAMPLE_COUNT: SampleCount = 4;

    fn new(event_loop: &os::EventLoop<ApplicationEvent>) -> Result<Self, ApplicationError> {
        let window = os::WindowBuilder::new()
            .with_inner_size(os::Size::Physical(os::PhysicalSize {
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

        let projection_transform = roe_math::ortographic_projection2(
            0.,
            window_size.width as f32,
            window_size.height as f32,
            0.,
        );

        let pipeline = roe_text::RenderPipeline::new(
            &instance,
            &roe_text::RenderPipelineDescriptor {
                sample_count: Self::SAMPLE_COUNT,
                ..roe_text::RenderPipelineDescriptor::default()
            },
        );

        let lib = FontLibrary::new()?;
        let face = Face::from_file(&lib, "roe_examples/data/fonts/Roboto-Regular.ttf", 0)?;
        let font = Font::new(&instance, &face, 11., &character_set::english())?;

        Ok(Self {
            window,
            instance,
            projection_transform,
            pipeline,
            font,
        })
    }
}

impl ApplicationState<ApplicationError, ApplicationEvent> for ApplicationImpl {
    fn on_resized(
        &mut self,
        wid: os::WindowId,
        size: os::PhysicalSize<u32>,
    ) -> Result<(), ApplicationError> {
        if wid == self.window.id() {
            self.window.update_buffer(&self.instance);
            self.projection_transform = roe_math::ortographic_projection2(
                0.,
                1f32.max(size.width as f32),
                1f32.max(size.height as f32),
                0.,
            );
        }
        Ok(())
    }

    fn on_variable_update(&mut self, _dt: std::time::Duration) -> Result<(), ApplicationError> {
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
                        &self.font,
                        "Lorem ipsum dolor sit amet",
                        &(self.projection_transform
                            * roe_math::translation2(&Vector2::new(100., 100.))),
                        &ColorF32::BLUE,
                    );
                }
                {
                    rpass.draw_text(
                        &self.pipeline,
                        &self.font,
                        "Hello world!",
                        &(self.projection_transform
                            * roe_math::translation2(&Vector2::new(300., 300.))),
                        &ColorF32::RED,
                    );
                }
            }

            cmd_sequence.submit(&self.instance);
            frame.present();
        }
        Ok(())
    }
}

fn main() {
    const FIXED_FRAMERATE: u64 = 30;
    const VARIABLE_FRAMERATE_CAP: u64 = 60;
    Application::new(FIXED_FRAMERATE, Some(VARIABLE_FRAMERATE_CAP))
        .run(|event_queue| Ok(Box::new(ApplicationImpl::new(event_queue)?)));
}
