use roe_app::{
    application::Application,
    event::{ControlFlow, EventHandler, EventLoop},
    window,
    window::{WindowBuilder, WindowId},
};

use roe_math::{
    conversion::convert,
    geometry2::{OrthographicProjection, Projective},
};

use roe_gfx::{
    core::{
        AddressMode, Canvas, CanvasWindow, CanvasWindowDescriptor, ColorF32, CommandSequence,
        FilterMode, Instance, InstanceDescriptor, RenderPassOperations, SampleCount, Sampler,
        SamplerDescriptor, Texture, TextureUsage, TextureViewDescriptor,
    },
    sprite,
    sprite::{MeshTemplates as SpriteMeshTemplates, Renderer as SpriteRenderer},
};

mod example_app;
use example_app::*;

#[derive(Debug)]
struct Sprite {
    uniform_constants: sprite::UniformConstants,
    mesh: sprite::Mesh,
}

#[derive(Debug)]
struct ApplicationImpl {
    window: CanvasWindow,
    instance: Instance,
    pipeline: sprite::RenderPipeline,
    projection_transform: Projective<f32>,
    sprites: Vec<Sprite>,
    color: ChangingColor,
}

impl ApplicationImpl {
    const SAMPLE_COUNT: SampleCount = 8;

    fn create_sprites(instance: &Instance) -> Vec<Sprite> {
        let image = image::open("roe_gfx/data/pictures/gioconda.jpg")
            .expect("Failed to load texture image")
            .into_rgba8();
        let sprite_texture = Texture::from_image(instance, &image, TextureUsage::SAMPLED)
            .create_view(&TextureViewDescriptor::default());

        vec![
            Sprite {
                uniform_constants: sprite::UniformConstants::new(
                    instance,
                    &sprite_texture,
                    &Sampler::new(&instance, &SamplerDescriptor::default()),
                ),
                mesh: sprite::Mesh::quad(
                    instance,
                    &sprite::Vertex::new([0., 0.], [0., 0.]),
                    &sprite::Vertex::new([400., 400.], [1., 1.]),
                ),
            },
            Sprite {
                uniform_constants: sprite::UniformConstants::new(
                    instance,
                    &sprite_texture,
                    &Sampler::new(
                        &instance,
                        &SamplerDescriptor {
                            mag_filter: FilterMode::Nearest,
                            min_filter: FilterMode::Linear,
                            mipmap_filter: FilterMode::Nearest,
                            ..SamplerDescriptor::default()
                        },
                    ),
                ),
                mesh: sprite::Mesh::quad(
                    instance,
                    &sprite::Vertex::new([400., 0.], [0., 0.]),
                    &sprite::Vertex::new([600., 200.], [0.5, 0.5]),
                ),
            },
            Sprite {
                uniform_constants: sprite::UniformConstants::new(
                    instance,
                    &sprite_texture,
                    &Sampler::new(
                        &instance,
                        &SamplerDescriptor {
                            mag_filter: FilterMode::Linear,
                            min_filter: FilterMode::Linear,
                            mipmap_filter: FilterMode::Linear,
                            ..SamplerDescriptor::default()
                        },
                    ),
                ),
                mesh: sprite::Mesh::quad(
                    instance,
                    &sprite::Vertex::new([800., 0.], [1., 0.]),
                    &sprite::Vertex::new([600., 200.], [0.5, 0.5]),
                ),
            },
            Sprite {
                uniform_constants: sprite::UniformConstants::new(
                    instance,
                    &sprite_texture,
                    &Sampler::new(
                        &instance,
                        &SamplerDescriptor {
                            mag_filter: FilterMode::Linear,
                            min_filter: FilterMode::Linear,
                            mipmap_filter: FilterMode::Linear,
                            ..SamplerDescriptor::default()
                        },
                    ),
                ),
                mesh: sprite::Mesh::quad(
                    instance,
                    &sprite::Vertex::new([400., 400.], [0., 1.]),
                    &sprite::Vertex::new([600., 200.], [0.5, 0.5]),
                ),
            },
            Sprite {
                uniform_constants: sprite::UniformConstants::new(
                    instance,
                    &sprite_texture,
                    &Sampler::new(
                        &instance,
                        &SamplerDescriptor {
                            mag_filter: FilterMode::Nearest,
                            min_filter: FilterMode::Linear,
                            mipmap_filter: FilterMode::Nearest,
                            ..SamplerDescriptor::default()
                        },
                    ),
                ),
                mesh: sprite::Mesh::quad(
                    instance,
                    &sprite::Vertex::new([600., 200.], [0.5, 0.5]),
                    &sprite::Vertex::new([800., 400.], [1., 1.]),
                ),
            },
            Sprite {
                uniform_constants: sprite::UniformConstants::new(
                    instance,
                    &sprite_texture,
                    &Sampler::new(
                        &instance,
                        &SamplerDescriptor {
                            address_mode_u: AddressMode::Repeat,
                            address_mode_v: AddressMode::ClampToEdge,
                            ..SamplerDescriptor::default()
                        },
                    ),
                ),
                mesh: sprite::Mesh::quad(
                    instance,
                    &sprite::Vertex::new([000., 400.], [-0.5, -0.5]),
                    &sprite::Vertex::new([400., 800.], [1.5, 1.5]),
                ),
            },
            Sprite {
                uniform_constants: sprite::UniformConstants::new(
                    instance,
                    &sprite_texture,
                    &Sampler::new(
                        &instance,
                        &SamplerDescriptor {
                            address_mode_u: AddressMode::MirrorRepeat,
                            address_mode_v: AddressMode::ClampToEdge,
                            ..SamplerDescriptor::default()
                        },
                    ),
                ),
                mesh: sprite::Mesh::quad(
                    instance,
                    &sprite::Vertex::new([800., 800.], [1.5, 1.5]),
                    &sprite::Vertex::new([400., 400.], [-0.5, -0.5]),
                ),
            },
        ]
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

        let pipeline = sprite::RenderPipeline::new(
            &instance,
            &sprite::RenderPipelineDescriptor {
                sample_count: Self::SAMPLE_COUNT,
                ..sprite::RenderPipelineDescriptor::default()
            },
        );

        let window_size = window.inner_size();

        let projection_transform = OrthographicProjection::new(
            0.,
            window_size.width as f32,
            window_size.height as f32,
            0.,
        )
        .to_projective();

        let sprites = Self::create_sprites(&instance);

        let color = ChangingColor::new(ColorF32::WHITE, ColorF32::WHITE);

        Ok(Self {
            window,
            instance,
            pipeline,
            projection_transform,
            sprites,
            color,
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

    fn on_variable_update(&mut self, dt: std::time::Duration) -> Result<ControlFlow, Self::Error> {
        self.color.update(dt);
        let push_constants = sprite::PushConstants::new(
            &convert(self.projection_transform),
            *self.color.current_color(),
        );

        let frame = self.window.current_frame()?;
        let mut cmd_sequence = CommandSequence::new(&self.instance);
        {
            let mut rpass = cmd_sequence.begin_render_pass(
                &frame,
                &self.pipeline.render_pass_requirements(),
                &RenderPassOperations::default(),
            );

            // Single draw
            // for sprite in &self.sprites {
            //     rpass.draw_sprite(
            //         &self.pipeline,
            //         &sprite.uniform_constants,
            //         &sprite.mesh,
            //         &push_constants,
            //         0..sprite.mesh.index_count(),
            //     );
            // }

            // Multiple draw using std::Vec
            for sprite in &self.sprites {
                rpass.draw_sprite_array(
                    &self.pipeline,
                    vec![(
                        &sprite.uniform_constants,
                        vec![(
                            &sprite.mesh,
                            vec![(&push_constants, vec![0..sprite.mesh.index_count()])],
                        )],
                    )],
                );
            }
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
