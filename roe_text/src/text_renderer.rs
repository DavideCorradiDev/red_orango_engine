extern crate freetype as ft;
extern crate harfbuzz_rs as hb;

use std::{fmt::Debug, mem::size_of};

use num_traits::identities::Zero;

pub use gfx::{MeshIndex, MeshIndexRange};
use roe_graphics as gfx;

use roe_math::{conversion::ToHomogeneousMatrix3, geometry2, geometry3};

use super::{i26dot6_to_fsize, Font, GlyphRenderingInfo};

fn as_push_constants_slice<T>(value: &T) -> &[u32] {
    let data: *const T = value;
    let data = data as *const u8;
    let data = unsafe { std::slice::from_raw_parts(data, size_of::<T>()) };
    bytemuck::cast_slice(&data)
}

#[repr(C, packed)]
#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
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
                    ty: gfx::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: gfx::TextureComponentType::Float,
                        dimension: gfx::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                gfx::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: gfx::ShaderStage::FRAGMENT,
                    ty: gfx::BindingType::Sampler { comparison: false },
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

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct RenderPipelineDescriptor {
    pub color_blend: gfx::BlendDescriptor,
    pub alpha_blend: gfx::BlendDescriptor,
    pub write_mask: gfx::ColorWrite,
    pub color_buffer_format: gfx::CanvasColorBufferFormat,
    pub sample_count: gfx::SampleCount,
}

impl Default for RenderPipelineDescriptor {
    fn default() -> Self {
        Self {
            color_blend: gfx::BlendDescriptor {
                src_factor: gfx::BlendFactor::SrcAlpha,
                dst_factor: gfx::BlendFactor::OneMinusSrcAlpha,
                operation: gfx::BlendOperation::Add,
            },
            alpha_blend: gfx::BlendDescriptor {
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
            gfx::include_spirv!("shaders/gen/spirv/text.vert.spv"),
        );
        let fs_module = gfx::ShaderModule::new(
            instance,
            gfx::include_spirv!("shaders/gen/spirv/text.frag.spv"),
        );
        let pipeline = gfx::RenderPipeline::new(
            instance,
            &gfx::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex_stage: gfx::ProgrammableStageDescriptor {
                    module: &vs_module,
                    entry_point: "main",
                },
                fragment_stage: Some(gfx::ProgrammableStageDescriptor {
                    module: &fs_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(gfx::RasterizationStateDescriptor {
                    front_face: gfx::FrontFace::Ccw,
                    cull_mode: gfx::CullMode::Back,
                    ..Default::default()
                }),
                primitive_topology: gfx::PrimitiveTopology::TriangleList,
                color_states: &[gfx::ColorStateDescriptor {
                    format: gfx::TextureFormat::from(desc.color_buffer_format),
                    color_blend: desc.color_blend.clone(),
                    alpha_blend: desc.alpha_blend.clone(),
                    write_mask: desc.write_mask,
                }],
                depth_stencil_state: None,
                vertex_state: gfx::VertexStateDescriptor {
                    index_format: gfx::IndexFormat::Uint16,
                    vertex_buffers: &[gfx::VertexBufferDescriptor {
                        stride: std::mem::size_of::<Vertex>() as gfx::BufferAddress,
                        step_mode: gfx::InputStepMode::Vertex,
                        attributes: &[
                            gfx::VertexAttributeDescriptor {
                                format: gfx::VertexFormat::Float2,
                                offset: 0,
                                shader_location: 0,
                            },
                            gfx::VertexAttributeDescriptor {
                                format: gfx::VertexFormat::Float3,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    }],
                },
                sample_count: desc.sample_count,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
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
        self.set_index_buffer(font.index_buffer().slice(..));
        self.set_vertex_buffer(0, font.vertex_buffer().slice(..));

        let pc = (
            transform.to_homogeneous3(),
            geometry3::HomogeneousVector::<f32>::zero(),
            color.clone(),
        );
        self.set_push_constants(gfx::ShaderStage::VERTEX, 0, as_push_constants_slice(&pc));

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
                as_push_constants_slice(&offset),
            );
            self.draw_indexed(index_range.clone(), 0, 0..1);

            cursor_pos.x = cursor_pos.x + i26dot6_to_fsize(position.x_advance);
            cursor_pos.y = cursor_pos.y + i26dot6_to_fsize(position.y_advance);
        }
    }
}
