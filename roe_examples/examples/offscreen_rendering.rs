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
    canvas: CanvasTexture,
    instance: Instance,
    shape2_pipeline: roe_shape2::RenderPipeline,
    triangle_mesh: roe_shape2::Mesh,
    sprite_pipeline: roe_sprite::RenderPipeline,
    quad_mesh: roe_sprite::Mesh,
    sprite_uniform_constants: roe_sprite::UniformConstants,
    current_angle: f32,
    color: ChangingColor,
}

impl ApplicationImpl {
    const SAMPLE_COUNT: SampleCount = 4;
    const SCREENSHOT_PATH: &'static str = "screenshot.png";

    pub fn update_angle(&mut self, dt: std::time::Duration) {
        const ANGULAR_SPEED: f32 = std::f32::consts::PI * 0.25;
        self.current_angle = self.current_angle + ANGULAR_SPEED * dt.as_secs_f32();
        while self.current_angle >= std::f32::consts::PI * 2. {
            self.current_angle = self.current_angle - std::f32::consts::PI * 2.;
        }
    }

    pub fn generate_triangle_push_constants(&self) -> roe_shape2::PushConstants {
        let projection_transform = OrthographicProjection::new(0., 1., 1., 0.).to_projective();
        let object_transform = Similarity::<f32>::from_parts(
            Translation::new(0.5, 0.5),
            UnitComplex::new(self.current_angle),
            1.,
        );
        roe_shape2::PushConstants::new(
            &convert(projection_transform * object_transform),
            *self.color.current_color(),
        )
    }

    pub fn generate_blit_push_constants(&self) -> roe_sprite::PushConstants {
        let projection_transform = OrthographicProjection::new(0., 1., 1., 0.).to_projective();
        roe_sprite::PushConstants::new(&convert(projection_transform), ColorF32::WHITE)
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
                &CanvasWindowDescriptor::default(),
            );
            (window, instance)
        };

        let canvas = CanvasTexture::new(
            &instance,
            &CanvasTextureDescriptor {
                size: Size::new(100, 100),
                sample_count: Self::SAMPLE_COUNT,
                color_buffer_descriptor: Some(CanvasTextureColorBufferDescriptor {
                    format: CanvasColorBufferFormat::Rgba8UnormSrgb,
                    usage: CanvasColorBufferUsage::TEXTURE_BINDING
                        | CanvasColorBufferUsage::COPY_SRC,
                }),
                ..CanvasTextureDescriptor::default()
            },
        );

        let shape2_pipeline = roe_shape2::RenderPipeline::new(
            &instance,
            &roe_shape2::RenderPipelineDescriptor {
                sample_count: Self::SAMPLE_COUNT,
                color_buffer_format: CanvasColorBufferFormat::Rgba8UnormSrgb,
                ..roe_shape2::RenderPipelineDescriptor::default()
            },
        );

        let triangle_mesh = roe_shape2::Mesh::new(
            &instance,
            &[
                roe_shape2::Vertex::new([-0.25, 0.25]),
                roe_shape2::Vertex::new([0.25, 0.25]),
                roe_shape2::Vertex::new([0., -0.25]),
            ],
            &[0, 1, 2],
        );

        let sprite_pipeline = roe_sprite::RenderPipeline::new(
            &instance,
            &roe_sprite::RenderPipelineDescriptor::default(),
        );

        let quad_mesh = roe_sprite::Mesh::quad(
            &instance,
            &roe_sprite::Vertex::new([0., 0.], [0., 0.]),
            &roe_sprite::Vertex::new([1., 1.], [2., 2.]),
        );

        let sampler = Sampler::new(
            &instance,
            &SamplerDescriptor {
                address_mode_u: AddressMode::MirrorRepeat,
                address_mode_v: AddressMode::MirrorRepeat,
                ..SamplerDescriptor::default()
            },
        );

        let canvas_texture_view = canvas
            .color_texture_view()
            .expect("The canvas color buffer doesn't exist");
        let sprite_uniform_constants =
            roe_sprite::UniformConstants::new(&instance, canvas_texture_view, &sampler);

        let color = ChangingColor::new(ColorF32::WHITE, ColorF32::WHITE);

        Ok(Self {
            window,
            canvas,
            instance,
            shape2_pipeline,
            triangle_mesh,
            sprite_pipeline,
            quad_mesh,
            sprite_uniform_constants,
            current_angle: 0.,
            color,
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
                if key == keyboard::KeyCode::Return {
                    let image = self
                        .canvas
                        .color_texture()
                        .unwrap()
                        .to_image(&self.instance);
                    image.save(Self::SCREENSHOT_PATH).unwrap();
                }
            }
        }
        Ok(ControlFlow::Continue)
    }

    fn on_event_loop_destroyed(&mut self) -> Result<ControlFlow, Self::Error> {
        if std::path::Path::new(Self::SCREENSHOT_PATH).exists() {
            std::fs::remove_file(Self::SCREENSHOT_PATH).unwrap();
        }
        Ok(ControlFlow::Continue)
    }

    fn on_resized(
        &mut self,
        wid: WindowId,
        _size: window::PhysicalSize<u32>,
    ) -> Result<ControlFlow, Self::Error> {
        if wid == self.window.id() {
            self.window.update_buffer(&self.instance);
        }
        Ok(ControlFlow::Continue)
    }

    fn on_variable_update(&mut self, dt: std::time::Duration) -> Result<ControlFlow, Self::Error> {
        self.color.update(dt);
        self.update_angle(dt);

        {
            // Render a triangle onto the canvas texture.
            let push_constants = self.generate_triangle_push_constants();
            let frame = self.canvas.current_frame()?;
            let mut cmd_sequence = CommandSequence::new(&self.instance);
            {
                let mut rpass = cmd_sequence.begin_render_pass(
                    &frame,
                    &self.shape2_pipeline.render_pass_requirements(),
                    &RenderPassOperations {
                        color_operations: vec![ColorOperations {
                            load: LoadOp::Clear(ColorF64::BLACK),
                            store: true,
                        }],
                        ..RenderPassOperations::default()
                    },
                );
                rpass.draw_shape2(
                    &self.shape2_pipeline,
                    &self.triangle_mesh,
                    &push_constants,
                    0..self.triangle_mesh.index_count(),
                );
            }
            cmd_sequence.submit(&self.instance);
            frame.present();
        }
        {
            // Render the canvas texture onto the canvas window.
            let push_constants = self.generate_blit_push_constants();
            let frame = self.window.current_frame()?;
            let mut cmd_sequence = CommandSequence::new(&self.instance);
            {
                let mut rpass = cmd_sequence.begin_render_pass(
                    &frame,
                    &self.sprite_pipeline.render_pass_requirements(),
                    &RenderPassOperations {
                        color_operations: vec![ColorOperations {
                            load: LoadOp::Clear(ColorF64::WHITE),
                            store: true,
                        }],
                        ..RenderPassOperations::default()
                    },
                );
                rpass.draw_sprite(
                    &self.sprite_pipeline,
                    &self.sprite_uniform_constants,
                    &self.quad_mesh,
                    &push_constants,
                    0..self.quad_mesh.index_count(),
                );
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
