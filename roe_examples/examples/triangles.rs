use std::iter::once;

use rand::Rng;

use roe_app::{
    application::Application,
    event::{mouse, ControlFlow, DeviceId, EventHandler, EventLoop},
    window,
    window::{WindowBuilder, WindowId},
};

use roe_math::{
    conversion::convert,
    geometry2::{OrthographicProjection, Point, Projective, Similarity, Translation, UnitComplex},
};

use roe_graphics::{
    Canvas, CanvasWindow, CanvasWindowDescriptor, ColorF32, CommandSequence, Instance,
    InstanceDescriptor, RenderPassOperations, SampleCount,
};

use roe_shape2 as shape2;
use shape2::Renderer as Shape2Renderer;

use roe_examples::*;

#[derive(Debug)]
struct ApplicationImpl {
    window: CanvasWindow,
    instance: Instance,
    pipeline: shape2::RenderPipeline,
    triangle_mesh: shape2::Mesh,
    saved_triangle_constants: Vec<shape2::PushConstants>,
    projection_transform: Projective<f32>,
    current_position: Point<f32>,
    current_angle: f32,
    current_scaling: f32,
    current_color: ColorF32,
}

impl ApplicationImpl {
    const SAMPLE_COUNT: SampleCount = 8;

    pub fn update_angle(&mut self, dt: std::time::Duration) {
        const ANGULAR_SPEED: f32 = std::f32::consts::PI * 0.25;
        self.current_angle = self.current_angle + ANGULAR_SPEED * dt.as_secs_f32();
        while self.current_angle >= std::f32::consts::PI * 2. {
            self.current_angle = self.current_angle - std::f32::consts::PI * 2.;
        }
    }

    pub fn generate_push_constant(&self) -> shape2::PushConstants {
        let object_transform = Similarity::<f32>::from_parts(
            Translation::new(self.current_position.x, self.current_position.y),
            UnitComplex::new(self.current_angle),
            self.current_scaling,
        );
        shape2::PushConstants::new(
            &convert(self.projection_transform * object_transform),
            self.current_color,
        )
    }
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

        let pipeline = shape2::RenderPipeline::new(
            &instance,
            &shape2::RenderPipelineDescriptor {
                sample_count: Self::SAMPLE_COUNT,
                ..shape2::RenderPipelineDescriptor::default()
            },
        );

        let triangle_mesh = shape2::Mesh::new(
            &instance,
            &[
                shape2::Vertex::new([-50., 50.]),
                shape2::Vertex::new([50., 50.]),
                shape2::Vertex::new([0., -50.]),
            ],
            &[0, 1, 2],
        );

        let window_size = window.inner_size();

        let projection_transform = OrthographicProjection::new(
            0.,
            window_size.width as f32,
            window_size.height as f32,
            0.,
        )
        .to_projective();

        let current_position = Point::from([
            window_size.width as f32 / 2.,
            window_size.height as f32 / 2.,
        ]);

        let current_color = ColorF32 {
            r: 1.,
            g: 1.,
            b: 1.,
            a: 0.75,
        };

        Ok(Self {
            window,
            instance,
            pipeline,
            triangle_mesh,
            saved_triangle_constants: Vec::new(),
            projection_transform,
            current_position,
            current_angle: 0.,
            current_scaling: 1.,
            current_color,
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

    fn on_cursor_moved(
        &mut self,
        wid: WindowId,
        _device_id: DeviceId,
        position: window::PhysicalPosition<f64>,
    ) -> Result<ControlFlow, Self::Error> {
        if wid == self.window.id() {
            self.current_position.x = position.x as f32;
            self.current_position.y = position.y as f32;
        }
        Ok(ControlFlow::Continue)
    }

    fn on_mouse_button_released(
        &mut self,
        wid: WindowId,
        _device_id: DeviceId,
        button: mouse::Button,
    ) -> Result<ControlFlow, Self::Error> {
        if wid == self.window.id() {
            if button == mouse::Button::Left {
                self.saved_triangle_constants
                    .push(self.generate_push_constant());
                let mut rng = rand::thread_rng();
                self.current_scaling = rng.gen_range(0.25, 4.);
                self.current_color.r = rng.gen_range(0., 1.);
                self.current_color.g = rng.gen_range(0., 1.);
                self.current_color.b = rng.gen_range(0., 1.);
            }
        }
        Ok(ControlFlow::Continue)
    }

    fn on_variable_update(&mut self, dt: std::time::Duration) -> Result<ControlFlow, Self::Error> {
        self.update_angle(dt);

        let mut draw_static_triangle_params =
            Vec::with_capacity(self.saved_triangle_constants.len());
        for saved_triangle_constant in self.saved_triangle_constants.iter() {
            draw_static_triangle_params.push((
                saved_triangle_constant,
                once(0..self.triangle_mesh.index_count()),
            ));
        }

        let current_triangle_constants = self.generate_push_constant();

        let frame = self.window.current_frame()?;
        let mut cmd_sequence = CommandSequence::new(&self.instance);

        {
            let mut rpass = cmd_sequence.begin_render_pass(
                &frame,
                &self.pipeline.render_pass_requirements(),
                &RenderPassOperations::default(),
            );
            rpass.draw_shape2_array(
                &self.pipeline,
                once((&self.triangle_mesh, draw_static_triangle_params)),
            );
        }

        {
            // Technically this could be done in the same render pass, just showing how to
            // combine multiple render passes keeping what was rendered in the previous one.
            let mut rpass = cmd_sequence.begin_render_pass(
                &frame,
                &self.pipeline.render_pass_requirements(),
                &RenderPassOperations {
                    color_operations: vec![roe_graphics::Operations {
                        load: roe_graphics::LoadOp::Load,
                        store: true,
                    }],
                    ..RenderPassOperations::default()
                },
            );
            rpass.draw_shape2(
                &self.pipeline,
                &self.triangle_mesh,
                &current_triangle_constants,
                0..self.triangle_mesh.index_count(),
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
