use roe_app::{Application, ApplicationState};

use roe_os as os;

use roe_math::HomogeneousMatrix2;

use roe_graphics::{
    AddressMode, Canvas, CanvasWindow, CanvasWindowDescriptor, ColorF32, CommandSequence,
    FilterMode, Instance, InstanceDescriptor, RenderPassOperations, SampleCount, Sampler,
    SamplerDescriptor, Texture, TextureUsage, TextureView, TextureViewDescriptor,
};

use roe_sprite::{MeshTemplates as SpriteMeshTemplates, Renderer as SpriteRenderer};

use roe_examples::*;

use std::rc::Rc;

#[derive(Debug)]
struct Sprite {
    uniform_constants: roe_sprite::UniformConstants,
    mesh: roe_sprite::Mesh,
}

#[derive(Debug)]
struct ApplicationImpl {
    window: CanvasWindow,
    instance: Rc<Instance>,
    pipeline: roe_sprite::RenderPipeline,
    projection_transform: HomogeneousMatrix2<f32>,
    sprites: Vec<Sprite>,
    color: ChangingColor,
    texture_view: TextureView,
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

        let pipeline = roe_sprite::RenderPipeline::new(
            &instance,
            &roe_sprite::RenderPipelineDescriptor {
                sample_count: Self::SAMPLE_COUNT,
                ..roe_sprite::RenderPipelineDescriptor::default()
            },
        );

        let window_size = window.inner_size();

        let projection_transform = roe_math::ortographic_projection2(
            0.,
            window_size.width as f32,
            window_size.height as f32,
            0.,
        );

        let texture = Texture::from_image(
            &instance,
            &image::open("roe_examples/data/pictures/gioconda.jpg")?.into_rgba8(),
            TextureUsage::TEXTURE_BINDING,
        );

        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let sprites = Self::create_sprites(&instance, &texture_view)?;

        let color = ChangingColor::new(ColorF32::WHITE, ColorF32::WHITE);

        Ok(Self {
            window,
            instance,
            pipeline,
            projection_transform,
            sprites,
            color,
            texture_view,
        })
    }

    fn create_sprites(
        instance: &Instance,
        sprite_texture: &TextureView,
    ) -> Result<Vec<Sprite>, ApplicationError> {
        Ok(vec![
            Sprite {
                uniform_constants: roe_sprite::UniformConstants::new(
                    instance,
                    &sprite_texture,
                    &Sampler::new(&instance, &SamplerDescriptor::default()),
                ),
                mesh: roe_sprite::Mesh::quad(
                    instance,
                    &roe_sprite::Vertex::new([0., 0.], [0., 0.]),
                    &roe_sprite::Vertex::new([400., 400.], [1., 1.]),
                ),
            },
            Sprite {
                uniform_constants: roe_sprite::UniformConstants::new(
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
                mesh: roe_sprite::Mesh::quad(
                    instance,
                    &roe_sprite::Vertex::new([400., 0.], [0., 0.]),
                    &roe_sprite::Vertex::new([600., 200.], [0.5, 0.5]),
                ),
            },
            Sprite {
                uniform_constants: roe_sprite::UniformConstants::new(
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
                mesh: roe_sprite::Mesh::quad(
                    instance,
                    &roe_sprite::Vertex::new([800., 0.], [1., 0.]),
                    &roe_sprite::Vertex::new([600., 200.], [0.5, 0.5]),
                ),
            },
            Sprite {
                uniform_constants: roe_sprite::UniformConstants::new(
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
                mesh: roe_sprite::Mesh::quad(
                    instance,
                    &roe_sprite::Vertex::new([400., 400.], [0., 1.]),
                    &roe_sprite::Vertex::new([600., 200.], [0.5, 0.5]),
                ),
            },
            Sprite {
                uniform_constants: roe_sprite::UniformConstants::new(
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
                mesh: roe_sprite::Mesh::quad(
                    instance,
                    &roe_sprite::Vertex::new([600., 200.], [0.5, 0.5]),
                    &roe_sprite::Vertex::new([800., 400.], [1., 1.]),
                ),
            },
            Sprite {
                uniform_constants: roe_sprite::UniformConstants::new(
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
                mesh: roe_sprite::Mesh::quad(
                    instance,
                    &roe_sprite::Vertex::new([000., 400.], [-0.5, -0.5]),
                    &roe_sprite::Vertex::new([400., 800.], [1.5, 1.5]),
                ),
            },
            Sprite {
                uniform_constants: roe_sprite::UniformConstants::new(
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
                mesh: roe_sprite::Mesh::quad(
                    instance,
                    &roe_sprite::Vertex::new([800., 800.], [1.5, 1.5]),
                    &roe_sprite::Vertex::new([400., 400.], [-0.5, -0.5]),
                ),
            },
        ])
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

    fn on_variable_update(&mut self, dt: std::time::Duration) -> Result<(), ApplicationError> {
        self.color.update(dt);
        let push_constants =
            roe_sprite::PushConstants::new(&self.projection_transform, *self.color.current_color());

        if let Some(frame) = self.window.current_frame()? {
            let mut cmd_sequence = CommandSequence::new(&self.instance);
            {
                let mut rpass = cmd_sequence.begin_render_pass(
                    &frame,
                    &self.pipeline.render_pass_requirements(),
                    &RenderPassOperations::default(),
                );

                // Single draw calls
                for sprite in &self.sprites {
                    rpass.draw_sprite(
                        &self.pipeline,
                        &sprite.uniform_constants,
                        &sprite.mesh,
                        &push_constants,
                        0..sprite.mesh.index_count(),
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
