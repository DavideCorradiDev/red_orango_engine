use std::iter::once;

use rand::Rng;

use roe_app::{Application, ApplicationState};

use roe_os as os;

use roe_math::{HomogeneousMatrix2, Rotation2, Vector2};

use roe_graphics::{
    Canvas, CanvasWindow, CanvasWindowDescriptor, ColorF32, CommandSequence, Instance,
    InstanceDescriptor, RenderPassOperations, SampleCount,
};

use roe_shape::Renderer as Shape2Renderer;

use roe_examples::*;

#[derive(Debug)]
struct ApplicationImpl {
    window: CanvasWindow,
    instance: Instance,
    pipeline: roe_shape::RenderPipeline,
    triangle_mesh: roe_shape::Mesh,
    saved_triangle_constants: Vec<roe_shape::PushConstants>,
    projection_transform: HomogeneousMatrix2<f32>,
    current_offset: Vector2<f32>,
    current_angle: f32,
    current_scaling: f32,
    current_color: ColorF32,
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
            (window, instance)
        };

        let pipeline = roe_shape::RenderPipeline::new(
            &instance,
            &roe_shape::RenderPipelineDescriptor {
                sample_count: Self::SAMPLE_COUNT,
                ..roe_shape::RenderPipelineDescriptor::default()
            },
        );

        let triangle_mesh = roe_shape::Mesh::new(
            &instance,
            &[
                roe_shape::Vertex::new([-50., 50.]),
                roe_shape::Vertex::new([50., 50.]),
                roe_shape::Vertex::new([0., -50.]),
            ],
            &[0, 1, 2],
        );

        let window_size = window.inner_size();

        let projection_transform = roe_math::ortographic_projection2(
            0.,
            window_size.width as f32,
            window_size.height as f32,
            0.,
        );

        let current_offset = Vector2::from([
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
            current_offset,
            current_angle: 0.,
            current_scaling: 1.,
            current_color,
        })
    }

    pub fn update_angle(&mut self, dt: std::time::Duration) {
        const ANGULAR_SPEED: f32 = std::f32::consts::PI * 0.25;
        self.current_angle = self.current_angle + ANGULAR_SPEED * dt.as_secs_f32();
        while self.current_angle >= std::f32::consts::PI * 2. {
            self.current_angle = self.current_angle - std::f32::consts::PI * 2.;
        }
    }

    pub fn generate_push_constant(&self) -> roe_shape::PushConstants {
        let object_transform = roe_math::translation2(&Vector2::from(self.current_offset))
            * roe_math::rotation2(&Rotation2::new(self.current_angle))
            * roe_math::scale2(&Vector2::new(self.current_scaling, self.current_scaling));
        roe_shape::PushConstants::new(
            &(self.projection_transform * object_transform),
            self.current_color,
        )
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

    fn on_scale_factor_changed<'a>(
        &mut self,
        wid: os::WindowId,
        _scale_factor: f64,
        size: &'a mut os::PhysicalSize<u32>,
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

    fn on_cursor_moved(
        &mut self,
        wid: os::WindowId,
        _device_id: os::DeviceId,
        position: os::PhysicalPosition<f64>,
    ) -> Result<(), ApplicationError> {
        if wid == self.window.id() {
            self.current_offset.x = position.x as f32;
            self.current_offset.y = position.y as f32;
        }
        Ok(())
    }

    fn on_mouse_button_released(
        &mut self,
        wid: os::WindowId,
        _device_id: os::DeviceId,
        button: os::MouseButton,
    ) -> Result<(), ApplicationError> {
        if wid == self.window.id() {
            if button == os::MouseButton::Left {
                self.saved_triangle_constants
                    .push(self.generate_push_constant());
                let mut rng = rand::thread_rng();
                self.current_scaling = rng.gen_range(0.25..4.0);
                self.current_color.r = rng.gen_range(0.0..1.0);
                self.current_color.g = rng.gen_range(0.0..1.0);
                self.current_color.b = rng.gen_range(0.0..1.0);
            }
        }
        Ok(())
    }

    fn on_variable_update(&mut self, dt: std::time::Duration) -> Result<(), ApplicationError> {
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

        if let Some(frame) = self.window.current_frame()? {
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
