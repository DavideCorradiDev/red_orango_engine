use std::{default::Default, iter::IntoIterator};

use num_traits::Zero;

use roe_math::{conversion::ToHomogeneousMatrix3, geometry2, geometry3};

use roe_graphics as gfx;

#[repr(C, packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
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

pub type MeshIndexRange = gfx::MeshIndexRange;
pub type MeshIndex = gfx::MeshIndex;
pub type Mesh = gfx::IndexedMesh<Vertex>;

pub trait MeshTemplates {
    fn rectangle(instance: &gfx::Instance, width: f32, height: f32) -> Self;
    fn quad(instance: &gfx::Instance, v1: &Vertex, v2: &Vertex) -> Self;
}

impl MeshTemplates for Mesh {
    fn rectangle(instance: &gfx::Instance, width: f32, height: f32) -> Self {
        let vertex_list = vec![
            Vertex::new([0., 0.], [0., 0.]),
            Vertex::new([0., height], [0., 1.]),
            Vertex::new([width, height], [1., 1.]),
            Vertex::new([width, 0.], [1., 0.]),
        ];
        let index_list = vec![0, 1, 3, 3, 1, 2];
        Self::new(instance, &vertex_list, &index_list)
    }

    fn quad(instance: &gfx::Instance, v1: &Vertex, v2: &Vertex) -> Self {
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

#[repr(C, packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PushConstants {
    transform: geometry3::HomogeneousMatrix<f32>,
    color: gfx::ColorF32,
}

impl PushConstants {
    pub fn new(transform: &geometry2::Transform<f32>, color: gfx::ColorF32) -> Self {
        Self {
            transform: transform.to_homogeneous3(),
            color,
        }
    }
}

unsafe impl bytemuck::Zeroable for PushConstants {
    fn zeroed() -> Self {
        Self {
            transform: geometry3::HomogeneousMatrix::zero(),
            color: gfx::ColorF32::default(),
        }
    }
}

unsafe impl bytemuck::Pod for PushConstants {}

#[derive(Debug)]
pub struct UniformConstants {
    texture_bind_group: gfx::TextureBindGroup,
}

impl UniformConstants {
    pub fn new(
        texture_bind_group: gfx::TextureBindGroup,
    ) -> Self {
        Self { texture_bind_group }
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

#[derive(Debug)]
pub struct RenderPipeline {
    pipeline: gfx::RenderPipeline,
    bind_group_layout: gfx::BindGroupLayout,
    sample_count: gfx::SampleCount,
    color_buffer_format: gfx::CanvasColorBufferFormat,
}

impl RenderPipeline {
    pub fn new(instance: &gfx::Instance, desc: &RenderPipelineDescriptor) -> Self {
        let bind_group_layout = gfx::TextureBindGroup::create_bind_group_layout(instance);
        let pipeline_layout = gfx::PipelineLayout::new(
            instance,
            &gfx::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[gfx::PushConstantRange {
                    stages: gfx::ShaderStage::VERTEX,
                    range: 0..std::mem::size_of::<PushConstants>() as u32,
                }],
            },
        );
        let vs_module = gfx::ShaderModule::new(
            instance,
            &gfx::include_spirv!("shaders/gen/spirv/sprite.vert.spv"),
        );
        let fs_module = gfx::ShaderModule::new(
            instance,
            &gfx::include_spirv!("shaders/gen/spirv/sprite.frag.spv"),
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
                                format: gfx::VertexFormat::Float32x2,
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
        RangeIt: IntoIterator<Item = gfx::MeshIndexRange>;
}

impl<'a> Renderer<'a> for gfx::RenderPass<'a> {
    fn draw_sprite(
        &mut self,
        pipeline: &'a RenderPipeline,
        uniform_constants: &'a UniformConstants,
        mesh: &'a Mesh,
        push_constants: &'a PushConstants,
        index_range: MeshIndexRange,
    ) {
        self.set_pipeline(&pipeline.pipeline);
        self.set_bind_group(0, &uniform_constants.texture_bind_group, &[]);
        self.set_index_buffer(mesh.index_buffer().slice(..), gfx::IndexFormat::Uint16);
        self.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
        self.set_push_constants(
            gfx::ShaderStage::VERTEX,
            0,
            gfx::utility::as_slice(push_constants),
        );
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
        RangeIt: IntoIterator<Item = gfx::MeshIndexRange>,
    {
        self.set_pipeline(&pipeline.pipeline);
        for (uc, meshes) in draw_commands.into_iter() {
            self.set_bind_group(0, &uc.texture_bind_group, &[]);
            for (mesh, pcs) in meshes.into_iter() {
                self.set_index_buffer(mesh.index_buffer().slice(..), gfx::IndexFormat::Uint16);
                self.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                for (pc, ranges) in pcs.into_iter() {
                    self.set_push_constants(
                        gfx::ShaderStage::VERTEX,
                        0,
                        gfx::utility::as_slice(pc),
                    );
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
        let instance = gfx::Instance::new(&gfx::InstanceDescriptor::default()).unwrap();
        let _pipeline = RenderPipeline::new(&instance, &RenderPipelineDescriptor::default());
    }
}
