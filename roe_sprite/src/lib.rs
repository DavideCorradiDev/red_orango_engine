use std::{default::Default, iter::IntoIterator};

use num_traits::Zero;

use roe_math::{conversion::ToHomogeneousMatrix3, geometry2, geometry3};

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

pub type MeshIndexRange = roe_graphics::MeshIndexRange;
pub type MeshIndex = roe_graphics::MeshIndex;
pub type Mesh = roe_graphics::IndexedMesh<Vertex>;

pub trait MeshTemplates {
    fn rectangle(instance: &roe_graphics::Instance, width: f32, height: f32) -> Self;
    fn quad(instance: &roe_graphics::Instance, v1: &Vertex, v2: &Vertex) -> Self;
}

impl MeshTemplates for Mesh {
    fn rectangle(instance: &roe_graphics::Instance, width: f32, height: f32) -> Self {
        let vertex_list = vec![
            Vertex::new([0., 0.], [0., 0.]),
            Vertex::new([0., height], [0., 1.]),
            Vertex::new([width, height], [1., 1.]),
            Vertex::new([width, 0.], [1., 0.]),
        ];
        let index_list = vec![0, 1, 3, 3, 1, 2];
        Self::new(instance, &vertex_list, &index_list)
    }

    fn quad(instance: &roe_graphics::Instance, v1: &Vertex, v2: &Vertex) -> Self {
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
    color: roe_graphics::ColorF32,
}

impl PushConstants {
    pub fn new(transform: &geometry2::Transform<f32>, color: roe_graphics::ColorF32) -> Self {
        Self {
            transform: transform.to_homogeneous3(),
            color,
        }
    }

    fn as_slice(&self) -> &[u8] {
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
            color: roe_graphics::ColorF32::default(),
        }
    }
}

unsafe impl bytemuck::Pod for PushConstants {}

fn bind_group_layout(instance: &roe_graphics::Instance) -> roe_graphics::BindGroupLayout {
    roe_graphics::BindGroupLayout::new(
        instance,
        &roe_graphics::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                roe_graphics::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: roe_graphics::ShaderStage::FRAGMENT,
                    ty: roe_graphics::BindingType::Texture {
                        multisampled: false,
                        sample_type: roe_graphics::TextureSampleType::Float { filterable: true },
                        view_dimension: roe_graphics::TextureViewDimension::D2,
                    },
                    count: None,
                },
                roe_graphics::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: roe_graphics::ShaderStage::FRAGMENT,
                    ty: roe_graphics::BindingType::Sampler {
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
    bind_group: roe_graphics::BindGroup,
}

impl UniformConstants {
    pub fn new(
        instance: &roe_graphics::Instance,
        texture: &roe_graphics::TextureView,
        sampler: &roe_graphics::Sampler,
    ) -> Self {
        let layout = bind_group_layout(instance);
        let bind_group = roe_graphics::BindGroup::new(
            instance,
            &roe_graphics::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    roe_graphics::BindGroupEntry {
                        binding: 0,
                        resource: roe_graphics::BindingResource::TextureView(texture),
                    },
                    roe_graphics::BindGroupEntry {
                        binding: 1,
                        resource: roe_graphics::BindingResource::Sampler(sampler),
                    },
                ],
            },
        );
        Self { bind_group }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RenderPipelineDescriptor {
    pub color_blend: roe_graphics::BlendDescriptor,
    pub alpha_blend: roe_graphics::BlendDescriptor,
    pub write_mask: roe_graphics::ColorWrite,
    pub color_buffer_format: roe_graphics::CanvasColorBufferFormat,
    pub sample_count: roe_graphics::SampleCount,
}

impl Default for RenderPipelineDescriptor {
    fn default() -> Self {
        Self {
            color_blend: roe_graphics::BlendDescriptor {
                src_factor: roe_graphics::BlendFactor::SrcAlpha,
                dst_factor: roe_graphics::BlendFactor::OneMinusSrcAlpha,
                operation: roe_graphics::BlendOperation::Add,
            },
            alpha_blend: roe_graphics::BlendDescriptor {
                src_factor: roe_graphics::BlendFactor::One,
                dst_factor: roe_graphics::BlendFactor::One,
                operation: roe_graphics::BlendOperation::Max,
            },
            write_mask: roe_graphics::ColorWrite::ALL,
            color_buffer_format: roe_graphics::CanvasColorBufferFormat::default(),
            sample_count: 1,
        }
    }
}

#[derive(Debug)]
pub struct RenderPipeline {
    pipeline: roe_graphics::RenderPipeline,
    bind_group_layout: roe_graphics::BindGroupLayout,
    sample_count: roe_graphics::SampleCount,
    color_buffer_format: roe_graphics::CanvasColorBufferFormat,
}

impl RenderPipeline {
    pub fn new(instance: &roe_graphics::Instance, desc: &RenderPipelineDescriptor) -> Self {
        let bind_group_layout = bind_group_layout(instance);
        let pipeline_layout = roe_graphics::PipelineLayout::new(
            instance,
            &roe_graphics::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[roe_graphics::PushConstantRange {
                    stages: roe_graphics::ShaderStage::VERTEX,
                    range: 0..std::mem::size_of::<PushConstants>() as u32,
                }],
            },
        );
        let vs_module = roe_graphics::ShaderModule::new(
            instance,
            &roe_graphics::include_spirv!("shaders/gen/spirv/sprite.vert.spv"),
        );
        let fs_module = roe_graphics::ShaderModule::new(
            instance,
            &roe_graphics::include_spirv!("shaders/gen/spirv/sprite.frag.spv"),
        );
        let pipeline = roe_graphics::RenderPipeline::new(
            instance,
            &roe_graphics::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: roe_graphics::VertexState {
                    module: &vs_module,
                    entry_point: "main",
                    buffers: &[roe_graphics::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as roe_graphics::BufferAddress,
                        step_mode: roe_graphics::InputStepMode::Vertex,
                        attributes: &[
                            roe_graphics::VertexAttribute {
                                format: roe_graphics::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            roe_graphics::VertexAttribute {
                                format: roe_graphics::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    }],
                },
                primitive: roe_graphics::PrimitiveState {
                    topology: roe_graphics::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: roe_graphics::FrontFace::Ccw,
                    cull_mode: Some(roe_graphics::Face::Back),
                    clamp_depth: false,
                    polygon_mode: roe_graphics::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: roe_graphics::MultisampleState {
                    count: desc.sample_count,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(roe_graphics::FragmentState {
                    module: &fs_module,
                    entry_point: "main",
                    targets: &[roe_graphics::ColorTargetState {
                        format: roe_graphics::TextureFormat::from(desc.color_buffer_format),
                        blend: Some(roe_graphics::BlendState {
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

    pub fn render_pass_requirements(&self) -> roe_graphics::RenderPassRequirements {
        roe_graphics::RenderPassRequirements {
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
        RangeIt: IntoIterator<Item = roe_graphics::MeshIndexRange>;
}

impl<'a> Renderer<'a> for roe_graphics::RenderPass<'a> {
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
        self.set_index_buffer(
            mesh.index_buffer().slice(..),
            roe_graphics::IndexFormat::Uint16,
        );
        self.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
        self.set_push_constants(
            roe_graphics::ShaderStage::VERTEX,
            0,
            push_constants.as_slice(),
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
        RangeIt: IntoIterator<Item = roe_graphics::MeshIndexRange>,
    {
        self.set_pipeline(&pipeline.pipeline);
        for (uc, meshes) in draw_commands.into_iter() {
            self.set_bind_group(0, &uc.bind_group, &[]);
            for (mesh, pcs) in meshes.into_iter() {
                self.set_index_buffer(
                    mesh.index_buffer().slice(..),
                    roe_graphics::IndexFormat::Uint16,
                );
                self.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                for (pc, ranges) in pcs.into_iter() {
                    self.set_push_constants(roe_graphics::ShaderStage::VERTEX, 0, pc.as_slice());
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
        let instance =
            roe_graphics::Instance::new(&roe_graphics::InstanceDescriptor::default()).unwrap();
        let _pipeline = RenderPipeline::new(&instance, &RenderPipelineDescriptor::default());
    }
}
