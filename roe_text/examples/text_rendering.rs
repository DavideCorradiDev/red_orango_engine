extern crate roe_text as text;

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
    InstanceCreationError, InstanceDescriptor, RenderPassOperations, SampleCount, SwapChainError,
};

use roe_text::FontError;

use text::Renderer as TextRenderer;

pub type ApplicationEvent = ();

#[derive(Debug)]
pub enum ApplicationError {
    WindowCreationFailed(window::OsError),
    InstanceCreationFailed(InstanceCreationError),
    RenderFrameCreationFailed(SwapChainError),
    FontCreationFailed(FontError),
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::WindowCreationFailed(e) => {
                write!(f, "Window creation failed ({})", e)
            }
            ApplicationError::InstanceCreationFailed(e) => {
                write!(f, "Instance creation failed ({})", e)
            }
            ApplicationError::RenderFrameCreationFailed(e) => {
                write!(f, "Render frame creation failed ({})", e)
            }
            ApplicationError::FontCreationFailed(e) => {
                write!(f, "Font creation failed ({})", e)
            }
        }
    }
}

impl std::error::Error for ApplicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ApplicationError::WindowCreationFailed(e) => Some(e),
            ApplicationError::InstanceCreationFailed(e) => Some(e),
            ApplicationError::RenderFrameCreationFailed(e) => Some(e),
            ApplicationError::FontCreationFailed(e) => Some(e),
        }
    }
}

impl From<window::OsError> for ApplicationError {
    fn from(e: window::OsError) -> Self {
        ApplicationError::WindowCreationFailed(e)
    }
}

impl From<InstanceCreationError> for ApplicationError {
    fn from(e: InstanceCreationError) -> Self {
        ApplicationError::InstanceCreationFailed(e)
    }
}

impl From<SwapChainError> for ApplicationError {
    fn from(e: SwapChainError) -> Self {
        ApplicationError::RenderFrameCreationFailed(e)
    }
}

impl From<FontError> for ApplicationError {
    fn from(e: FontError) -> Self {
        ApplicationError::FontCreationFailed(e)
    }
}

#[derive(Debug)]
struct ApplicationImpl {
    window: CanvasWindow,
    instance: Instance,
    projection_transform: Projective<f32>,
    pipeline: text::RenderPipeline,
    font_lib: text::FontLibrary,
    face: text::Face,
    font: text::Font,
}

impl ApplicationImpl {
    const SAMPLE_COUNT: SampleCount = 8;
    const FONT_PATH: &'static str = "roe_text/data/fonts/Roboto-Regular.ttf";
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

        let window_size = window.inner_size();

        let projection_transform = OrthographicProjection::new(
            0.,
            window_size.width as f32,
            window_size.height as f32,
            0.,
        )
        .to_projective();

        let pipeline = text::RenderPipeline::new(
            &instance,
            &text::RenderPipelineDescriptor {
                sample_count: Self::SAMPLE_COUNT,
                ..text::RenderPipelineDescriptor::default()
            },
        );

        let font_lib = text::FontLibrary::new()?;
        let face = text::Face::from_file(&font_lib, Self::FONT_PATH, 0)?;
        let font = text::Font::new(
            &instance,
            &face,
            10.,
            text::character_set::english().as_slice(),
        )?;

        Ok(Self {
            window,
            instance,
            projection_transform,
            pipeline,
            font_lib,
            face,
            font,
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
        let frame = self.window.current_frame()?;
        let mut cmd_sequence = CommandSequence::new(&self.instance);

        {
            let mut rpass = cmd_sequence.begin_render_pass(
                &frame,
                &self.pipeline.render_pass_requirements(),
                &RenderPassOperations::default(),
            );
            rpass.draw_text(
                &self.pipeline,
                &self.font,
                "Lorem ipsum dolor sit amet",
                &convert(self.projection_transform * Translation::new(100., 100.)),
                &ColorF32::BLUE,
            );
            rpass.draw_text(
                &self.pipeline,
                &self.font,
                "Hello world!",
                &convert(self.projection_transform * Translation::new(300., 300.)),
                &ColorF32::RED,
            );
        }

        cmd_sequence.submit(&self.instance);
        Ok(ControlFlow::Continue)
    }
}

fn main() {
    const FIXED_FRAMERATE: u64 = 30;
    const VARIABLE_FRAMERATE_CAP: u64 = 60;
    Application::<ApplicationImpl, _, _>::new(FIXED_FRAMERATE, Some(VARIABLE_FRAMERATE_CAP)).run();
}
