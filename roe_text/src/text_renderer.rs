extern crate freetype as ft;
extern crate harfbuzz_rs as hb;

use std::{fmt::Debug, mem::size_of};

use num_traits::identities::Zero;

pub use gfx::{MeshIndex, MeshIndexRange};
use roe_graphics as gfx;

use roe_math::{conversion::ToHomogeneousMatrix3, geometry2, geometry3};

use super::{i26dot6_to_fsize, Font, GlyphRenderingInfo};

#[repr(C, packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vertex {
    position: [f32; 2],
    texture_coordinates: [f32; 3],
}

impl Vertex {
    pub fn new(position: [f32; 2], texture_coordinates: [f32; 3]) -> Self {
        Self {
            position,
            texture_coordinates,
        }
    }
}

unsafe impl bytemuck::Zeroable for Vertex {
    fn zeroed() -> Self {
        Self::new([0., 0.], [0., 0., 0.])
    }
}

unsafe impl bytemuck::Pod for Vertex {}

pub type Mesh = gfx::IndexedMesh<Vertex>;

fn bind_group_layout(instance: &gfx::Instance) -> gfx::BindGroupLayout {
    gfx::BindGroupLayout::new(
        instance,
        &gfx::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                gfx::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: gfx::ShaderStage::FRAGMENT,
                    ty: gfx::BindingType::Texture {
                        multisampled: false,
                        sample_type: gfx::TextureSampleType::Float { filterable: true },
                        view_dimension: gfx::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                gfx::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: gfx::ShaderStage::FRAGMENT,
                    ty: gfx::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
            ],
        },
    )
}

#[derive(Debug)]
pub struct UniformConstants {
    bind_group: gfx::BindGroup,
}

impl UniformConstants {
    pub fn new(
        instance: &gfx::Instance,
        texture: &gfx::TextureView,
        sampler: &gfx::Sampler,
    ) -> Self {
        let layout = bind_group_layout(instance);
        let bind_group = gfx::BindGroup::new(
            instance,
            &gfx::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    gfx::BindGroupEntry {
                        binding: 0,
                        resource: gfx::BindingResource::TextureView(texture),
                    },
                    gfx::BindGroupEntry {
                        binding: 1,
                        resource: gfx::BindingResource::Sampler(sampler),
                    },
                ],
            },
        );
        Self { bind_group }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RenderPipelineDescriptor {
    pub color_blend: gfx::BlendComponent,
    pub alpha_blend: gfx::BlendComponent,
    pub write_mask: gfx::ColorWrite,
    pub color_buffer_format: gfx::CanvasColorBufferFormat,
    pub sample_count: gfx::SampleCount,
}

impl Default for RenderPipelineDescriptor {
    fn default() -> Self {
        Self {
            color_blend: gfx::BlendComponent {
                src_factor: gfx::BlendFactor::SrcAlpha,
                dst_factor: gfx::BlendFactor::OneMinusSrcAlpha,
                operation: gfx::BlendOperation::Add,
            },
            alpha_blend: gfx::BlendComponent {
                src_factor: gfx::BlendFactor::One,
                dst_factor: gfx::BlendFactor::One,
                operation: gfx::BlendOperation::Max,
            },
            write_mask: gfx::ColorWrite::ALL,
            color_buffer_format: gfx::CanvasColorBufferFormat::default(),
            sample_count: 1,
        }
    }
}

const PC_TRANSFORM_MEM_OFFSET: u32 = 0;
const PC_GLYPH_OFFSET_MEM_OFFSET: u32 =
    PC_TRANSFORM_MEM_OFFSET + size_of::<geometry3::HomogeneousMatrix<f32>>() as u32;
const PC_COLOR_MEM_OFFSET: u32 =
    PC_GLYPH_OFFSET_MEM_OFFSET + size_of::<geometry3::HomogeneousVector<f32>>() as u32;
const PC_SIZE: u32 = PC_COLOR_MEM_OFFSET + size_of::<gfx::ColorF32>() as u32;

#[derive(Debug)]
pub struct RenderPipeline {
    pipeline: gfx::RenderPipeline,
    bind_group_layout: gfx::BindGroupLayout,
    sample_count: gfx::SampleCount,
    color_buffer_format: gfx::CanvasColorBufferFormat,
}

impl RenderPipeline {
    pub fn new(instance: &gfx::Instance, desc: &RenderPipelineDescriptor) -> Self {
        let bind_group_layout = bind_group_layout(instance);
        let pipeline_layout = gfx::PipelineLayout::new(
            instance,
            &gfx::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[gfx::PushConstantRange {
                    stages: gfx::ShaderStage::VERTEX,
                    range: 0..PC_SIZE,
                }],
            },
        );
        let vs_module = gfx::ShaderModule::new(
            instance,
            &gfx::include_spirv!("shaders/gen/spirv/text.vert.spv"),
        );
        let fs_module = gfx::ShaderModule::new(
            instance,
            &gfx::include_spirv!("shaders/gen/spirv/text.frag.spv"),
        );
        let pipeline = gfx::RenderPipeline::new(
            instance,
            &gfx::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: gfx::VertexState {
                    module: &vs_module,
                    entry_point: "main",
                    buffers: &[gfx::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as gfx::BufferAddress,
                        step_mode: gfx::VertexStepMode::Vertex,
                        attributes: &[
                            gfx::VertexAttribute {
                                format: gfx::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            gfx::VertexAttribute {
                                format: gfx::VertexFormat::Float32x3,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    }],
                },
                primitive: gfx::PrimitiveState {
                    topology: gfx::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: gfx::FrontFace::Ccw,
                    cull_mode: Some(gfx::Face::Back),
                    clamp_depth: false,
                    polygon_mode: gfx::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: gfx::MultisampleState {
                    count: desc.sample_count,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(gfx::FragmentState {
                    module: &fs_module,
                    entry_point: "main",
                    targets: &[gfx::ColorTargetState {
                        format: gfx::TextureFormat::from(desc.color_buffer_format),
                        blend: Some(gfx::BlendState {
                            color: desc.color_blend.clone(),
                            alpha: desc.alpha_blend.clone(),
                        }),
                        write_mask: desc.write_mask,
                    }],
                }),
            },
        );
        Self {
            pipeline,
            bind_group_layout,
            sample_count: desc.sample_count,
            color_buffer_format: desc.color_buffer_format,
        }
    }

    pub fn render_pass_requirements(&self) -> gfx::RenderPassRequirements {
        gfx::RenderPassRequirements {
            sample_count: self.sample_count,
            color_buffer_formats: vec![self.color_buffer_format],
            depth_stencil_buffer_format: None,
        }
    }
}

pub trait Renderer<'a> {
    fn draw_text(
        &mut self,
        pipeline: &'a RenderPipeline,
        font: &'a Font,
        text: &str,
        transform: &geometry2::Transform<f32>,
        color: &gfx::ColorF32,
    );
}

impl<'a> Renderer<'a> for gfx::RenderPass<'a> {
    fn draw_text(
        &mut self,
        pipeline: &'a RenderPipeline,
        font: &'a Font,
        text: &str,
        transform: &geometry2::Transform<f32>,
        color: &gfx::ColorF32,
    ) {
        let shaping_output = font.shape_text(text);
        let positions = shaping_output.get_glyph_positions();
        let infos = shaping_output.get_glyph_infos();

        self.set_pipeline(&pipeline.pipeline);
        self.set_bind_group(0, &font.uniform_constants().bind_group, &[]);
        self.set_index_buffer(font.index_buffer().slice(..), gfx::IndexFormat::Uint16);
        self.set_vertex_buffer(0, font.vertex_buffer().slice(..));

        let pc = (
            transform.to_homogeneous3(),
            geometry3::HomogeneousVector::<f32>::zero(),
            color.clone(),
        );
        self.set_push_constants(gfx::ShaderStage::VERTEX, 0, gfx::utility::as_slice(&pc));

        let mut cursor_pos = geometry2::HomogeneousVector::<f32>::zero();
        for (position, info) in positions.iter().zip(infos) {
            let GlyphRenderingInfo {
                index_range,
                bearing,
            } = font.glyph_rendering_info(info.codepoint).clone();

            let mut offset = cursor_pos;
            offset.x = offset.x + bearing.x + i26dot6_to_fsize(position.x_offset);
            offset.y = offset.y + bearing.y + i26dot6_to_fsize(position.y_offset);

            self.set_push_constants(
                gfx::ShaderStage::VERTEX,
                PC_GLYPH_OFFSET_MEM_OFFSET,
                gfx::utility::as_slice(&offset),
            );
            self.draw_indexed(index_range.clone(), 0, 0..1);

            cursor_pos.x = cursor_pos.x + i26dot6_to_fsize(position.x_advance);
            cursor_pos.y = cursor_pos.y + i26dot6_to_fsize(position.y_advance);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{character_set, Face, Font, FontLibrary},
        *,
    };
    use galvanic_assert::{matchers::*, *};
    use gfx::Canvas;
    use roe_math::{conversion::convert, geometry2 as geo};

    #[test]
    #[serial_test::serial]
    fn creation() {
        let instance = gfx::Instance::new(&gfx::InstanceDescriptor::default()).unwrap();
        let _pipeline = RenderPipeline::new(&instance, &RenderPipelineDescriptor::default());
    }

    #[test]
    #[serial_test::serial]
    fn draw_text() {
        let instance = gfx::Instance::new(&gfx::InstanceDescriptor::default()).unwrap();
        let mut canvas = gfx::CanvasTexture::new(
            &instance,
            &gfx::CanvasTextureDescriptor {
                size: gfx::CanvasSize::new(300, 300),
                sample_count: 1,
                color_buffer_descriptor: Some(gfx::CanvasTextureColorBufferDescriptor {
                    format: gfx::CanvasColorBufferFormat::Rgba8Unorm,
                    usage: gfx::CanvasColorBufferUsage::COPY_SRC,
                }),
                depth_stencil_buffer_format: None,
            },
        );
        let pipeline = RenderPipeline::new(
            &instance,
            &RenderPipelineDescriptor {
                color_buffer_format: gfx::CanvasColorBufferFormat::Rgba8Unorm,
                ..RenderPipelineDescriptor::default()
            },
        );

        let font_lib = FontLibrary::new().unwrap();
        let face = Face::from_file(&font_lib, "data/fonts/Roboto-Regular.ttf", 0).unwrap();
        let font = Font::new(&instance, &face, 10., character_set::english().as_slice()).unwrap();

        let projection_transform =
            geo::OrthographicProjection::new(0., 300., 300., 0.).to_projective();

        {
            let frame = canvas.current_frame().unwrap().unwrap();
            let mut cmd_sequence = gfx::CommandSequence::new(&instance);
            {
                let mut rpass = cmd_sequence.begin_render_pass(
                    &frame,
                    &pipeline.render_pass_requirements(),
                    &gfx::RenderPassOperations::default(),
                );
                rpass.draw_text(
                    &pipeline,
                    &font,
                    "Lorem ipsum dolor sit amet",
                    &convert(projection_transform * geo::Translation::new(10., 60.)),
                    &gfx::ColorF32::BLUE,
                );
                rpass.draw_text(
                    &pipeline,
                    &font,
                    "Hello world!",
                    &convert(projection_transform * geo::Translation::new(30., 150.)),
                    &gfx::ColorF32::RED,
                );
            }
            cmd_sequence.submit(&instance);
            frame.present();
        }
        let expected_image = image::load(
            std::io::BufReader::new(std::fs::File::open("data/pictures/test_result.png").unwrap()),
            image::ImageFormat::Png,
        )
        .unwrap();
        assert_that!(
            &expected_image,
            is_variant!(image::DynamicImage::ImageRgba8)
        );

        if let image::DynamicImage::ImageRgba8(expected_image) = expected_image {
            let result_image = canvas.color_texture().unwrap().to_image(&instance);

            expect_that!(&result_image, eq(expected_image));
        }
    }
}
