use std::{default::Default, iter::IntoIterator};

use num_traits::Zero;

use roe_math::{conversion::ToHomogeneousMatrix3, geometry2, geometry3};

use crate::core;

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Vertex {
    pub position: [f32; 2],
    pub texture_coordinates: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 2], texture_coordinates: [f32; 2]) -> Self {
        Self {
            position,
            texture_coordinates,
        }
    }

    pub fn from_points(
        position: &geometry2::Point<f32>,
        texture_coordinates: &geometry2::Point<f32>,
    ) -> Self {
        Self {
            position: [position.x, position.y],
            texture_coordinates: [texture_coordinates.x, texture_coordinates.y],
        }
    }
}

unsafe impl bytemuck::Zeroable for Vertex {
    fn zeroed() -> Self {
        Self::new([0., 0.], [0., 0.])
    }
}

unsafe impl bytemuck::Pod for Vertex {}

pub type MeshIndexRange = core::MeshIndexRange;
pub type MeshIndex = core::MeshIndex;
pub type Mesh = core::IndexedMesh<Vertex>;

pub trait MeshTemplates {
    fn rectangle(instance: &core::Instance, width: f32, height: f32) -> Self;
    fn quad(instance: &core::Instance, v1: &Vertex, v2: &Vertex) -> Self;
}

impl MeshTemplates for Mesh {
    fn rectangle(instance: &core::Instance, width: f32, height: f32) -> Self {
        let vertex_list = vec![
            Vertex::new([0., 0.], [0., 0.]),
            Vertex::new([0., height], [0., 1.]),
            Vertex::new([width, height], [1., 1.]),
            Vertex::new([width, 0.], [1., 0.]),
        ];
        let index_list = vec![0, 1, 3, 3, 1, 2];
        Self::new(instance, &vertex_list, &index_list)
    }

    fn quad(instance: &core::Instance, v1: &Vertex, v2: &Vertex) -> Self {
        let (lp, rp, lt, rt) = {
            if v1.position[0] <= v2.position[0] {
                (
                    v1.position[0],
                    v2.position[0],
                    v1.texture_coordinates[0],
                    v2.texture_coordinates[0],
                )
            } else {
                (
                    v2.position[0],
                    v1.position[0],
                    v2.texture_coordinates[0],
                    v1.texture_coordinates[0],
                )
            }
        };
        let (tp, bp, tt, bt) = {
            if v1.position[1] <= v2.position[1] {
                (
                    v1.position[1],
                    v2.position[1],
                    v1.texture_coordinates[1],
                    v2.texture_coordinates[1],
                )
            } else {
                (
                    v2.position[1],
                    v1.position[1],
                    v2.texture_coordinates[1],
                    v1.texture_coordinates[1],
                )
            }
        };

        let vertex_list = vec![
            Vertex::new([lp, tp], [lt, tt]),
            Vertex::new([lp, bp], [lt, bt]),
            Vertex::new([rp, bp], [rt, bt]),
            Vertex::new([rp, tp], [rt, tt]),
        ];
        let index_list = vec![0, 1, 3, 3, 1, 2];
        Self::new(instance, &vertex_list, &index_list)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct PushConstants {
    transform: geometry3::HomogeneousMatrix<f32>,
    color: core::ColorF32,
}

impl PushConstants {
    pub fn new(transform: &geometry2::Transform<f32>, color: core::ColorF32) -> Self {
        Self {
            transform: transform.to_homogeneous3(),
            color,
        }
    }

    fn as_slice(&self) -> &[u32] {
        let pc: *const PushConstants = self;
        let pc: *const u8 = pc as *const u8;
        let data = unsafe { std::slice::from_raw_parts(pc, std::mem::size_of::<PushConstants>()) };
        bytemuck::cast_slice(&data)
    }
}

unsafe impl bytemuck::Zeroable for PushConstants {
    fn zeroed() -> Self {
        Self {
            transform: geometry3::HomogeneousMatrix::zero(),
            color: core::ColorF32::default(),
        }
    }
}

unsafe impl bytemuck::Pod for PushConstants {}

fn bind_group_layout(instance: &core::Instance) -> core::BindGroupLayout {
    core::BindGroupLayout::new(
        instance,
        &core::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                core::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: core::ShaderStage::FRAGMENT,
                    ty: core::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: core::TextureComponentType::Float,
                        dimension: core::TextureViewDimension::D2,
                    },
                    count: None,
                },
                core::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: core::ShaderStage::FRAGMENT,
                    ty: core::BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
        },
    )
}

#[derive(Debug)]
pub struct UniformConstants {
    bind_group: core::BindGroup,
}

impl UniformConstants {
    pub fn new(
        instance: &core::Instance,
        texture: &core::TextureView,
        sampler: &core::Sampler,
    ) -> Self {
        let layout = bind_group_layout(instance);
        let bind_group = core::BindGroup::new(
            instance,
            &core::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    core::BindGroupEntry {
                        binding: 0,
                        resource: core::BindingResource::TextureView(texture),
                    },
                    core::BindGroupEntry {
                        binding: 1,
                        resource: core::BindingResource::Sampler(sampler),
                    },
                ],
            },
        );
        Self { bind_group }
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct RenderPipelineDescriptor {
    pub color_blend: core::BlendDescriptor,
    pub alpha_blend: core::BlendDescriptor,
    pub write_mask: core::ColorWrite,
    pub color_buffer_format: core::CanvasColorBufferFormat,
    pub sample_count: core::SampleCount,
}

impl Default for RenderPipelineDescriptor {
    fn default() -> Self {
        Self {
            color_blend: core::BlendDescriptor {
                src_factor: core::BlendFactor::SrcAlpha,
                dst_factor: core::BlendFactor::OneMinusSrcAlpha,
                operation: core::BlendOperation::Add,
            },
            alpha_blend: core::BlendDescriptor {
                src_factor: core::BlendFactor::One,
                dst_factor: core::BlendFactor::One,
                operation: core::BlendOperation::Max,
            },
            write_mask: core::ColorWrite::ALL,
            color_buffer_format: core::CanvasColorBufferFormat::default(),
            sample_count: 1,
        }
    }
}

#[derive(Debug)]
pub struct RenderPipeline {
    pipeline: core::RenderPipeline,
    bind_group_layout: core::BindGroupLayout,
    sample_count: core::SampleCount,
    color_buffer_format: core::CanvasColorBufferFormat,
}

impl RenderPipeline {
    pub fn new(instance: &core::Instance, desc: &RenderPipelineDescriptor) -> Self {
        let bind_group_layout = bind_group_layout(instance);
        let pipeline_layout = core::PipelineLayout::new(
            instance,
            &core::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[core::PushConstantRange {
                    stages: core::ShaderStage::VERTEX,
                    range: 0..std::mem::size_of::<PushConstants>() as u32,
                }],
            },
        );
        let vs_module = core::ShaderModule::new(
            instance,
            core::include_spirv!("shaders/gen/spirv/sprite.vert.spv"),
        );
        let fs_module = core::ShaderModule::new(
            instance,
            core::include_spirv!("shaders/gen/spirv/sprite.frag.spv"),
        );
        let pipeline = core::RenderPipeline::new(
            instance,
            &core::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex_stage: core::ProgrammableStageDescriptor {
                    module: &vs_module,
                    entry_point: "main",
                },
                fragment_stage: Some(core::ProgrammableStageDescriptor {
                    module: &fs_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(core::RasterizationStateDescriptor {
                    front_face: core::FrontFace::Ccw,
                    cull_mode: core::CullMode::Back,
                    ..Default::default()
                }),
                primitive_topology: core::PrimitiveTopology::TriangleList,
                color_states: &[core::ColorStateDescriptor {
                    format: core::TextureFormat::from(desc.color_buffer_format),
                    color_blend: desc.color_blend.clone(),
                    alpha_blend: desc.alpha_blend.clone(),
                    write_mask: desc.write_mask,
                }],
                depth_stencil_state: None,
                vertex_state: core::VertexStateDescriptor {
                    index_format: core::IndexFormat::Uint16,
                    vertex_buffers: &[core::VertexBufferDescriptor {
                        stride: std::mem::size_of::<Vertex>() as core::BufferAddress,
                        step_mode: core::InputStepMode::Vertex,
                        attributes: &[
                            core::VertexAttributeDescriptor {
                                format: core::VertexFormat::Float2,
                                offset: 0,
                                shader_location: 0,
                            },
                            core::VertexAttributeDescriptor {
                                format: core::VertexFormat::Float2,
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

    pub fn render_pass_requirements(&self) -> core::RenderPassRequirements {
        core::RenderPassRequirements {
            sample_count: self.sample_count,
            color_buffer_formats: vec![self.color_buffer_format],
            depth_stencil_buffer_format: None,
        }
    }
}

pub trait Renderer<'a> {
    fn draw_sprite(
        &mut self,
        pipeline: &'a RenderPipeline,
        uniform_constants: &'a UniformConstants,
        mesh: &'a Mesh,
        push_constants: &'a PushConstants,
        index_range: MeshIndexRange,
    );

    fn draw_sprite_array<UcIt, MeshIt, PcIt, RangeIt>(
        &mut self,
        pipeline: &'a RenderPipeline,
        draw_commands: UcIt,
    ) where
        UcIt: IntoIterator<Item = (&'a UniformConstants, MeshIt)>,
        MeshIt: IntoIterator<Item = (&'a Mesh, PcIt)>,
        PcIt: IntoIterator<Item = (&'a PushConstants, RangeIt)>,
        RangeIt: IntoIterator<Item = core::MeshIndexRange>;
}

impl<'a> Renderer<'a> for core::RenderPass<'a> {
    fn draw_sprite(
        &mut self,
        pipeline: &'a RenderPipeline,
        uniform_constants: &'a UniformConstants,
        mesh: &'a Mesh,
        push_constants: &'a PushConstants,
        index_range: MeshIndexRange,
    ) {
        self.set_pipeline(&pipeline.pipeline);
        self.set_bind_group(0, &uniform_constants.bind_group, &[]);
        self.set_index_buffer(mesh.index_buffer().slice(..));
        self.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
        self.set_push_constants(core::ShaderStage::VERTEX, 0, push_constants.as_slice());
        self.draw_indexed(index_range, 0, 0..1);
    }

    fn draw_sprite_array<UcIt, MeshIt, PcIt, RangeIt>(
        &mut self,
        pipeline: &'a RenderPipeline,
        draw_commands: UcIt,
    ) where
        UcIt: IntoIterator<Item = (&'a UniformConstants, MeshIt)>,
        MeshIt: IntoIterator<Item = (&'a Mesh, PcIt)>,
        PcIt: IntoIterator<Item = (&'a PushConstants, RangeIt)>,
        RangeIt: IntoIterator<Item = core::MeshIndexRange>,
    {
        self.set_pipeline(&pipeline.pipeline);
        for (uc, meshes) in draw_commands.into_iter() {
            self.set_bind_group(0, &uc.bind_group, &[]);
            for (mesh, pcs) in meshes.into_iter() {
                self.set_index_buffer(mesh.index_buffer().slice(..));
                self.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                for (pc, ranges) in pcs.into_iter() {
                    self.set_push_constants(core::ShaderStage::VERTEX, 0, pc.as_slice());
                    for range in ranges.into_iter() {
                        self.draw_indexed(range, 0, 0..1);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn creation() {
        let instance = core::Instance::new(&core::InstanceDescriptor::default()).unwrap();
        let _pipeline = RenderPipeline::new(&instance, &RenderPipelineDescriptor::default());
    }
}
