use std::{default::Default, iter::IntoIterator};

use num_traits::Zero;

use roe_math::{conversion::ToHomogeneousMatrix3, geometry2, geometry3};

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 2]) -> Self {
        Self { position }
    }

    pub fn from_points(position: &geometry2::Point<f32>) -> Self {
        Self {
            position: [position.x, position.y],
        }
    }
}

unsafe impl bytemuck::Zeroable for Vertex {
    fn zeroed() -> Self {
        Self::new([0., 0.])
    }
}

unsafe impl bytemuck::Pod for Vertex {}

pub type MeshIndexRange = roe_graphics::MeshIndexRange;
pub type MeshIndex = roe_graphics::MeshIndex;
pub type Mesh = roe_graphics::IndexedMesh<Vertex>;

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
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
            color: roe_graphics::ColorF32::default(),
        }
    }
}

unsafe impl bytemuck::Pod for PushConstants {}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
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
    sample_count: roe_graphics::SampleCount,
    color_buffer_format: roe_graphics::CanvasColorBufferFormat,
}

impl RenderPipeline {
    pub fn new(instance: &roe_graphics::Instance, desc: &RenderPipelineDescriptor) -> Self {
        let pipeline_layout = roe_graphics::PipelineLayout::new(
            &instance,
            &roe_graphics::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[roe_graphics::PushConstantRange {
                    stages: roe_graphics::ShaderStage::VERTEX,
                    range: 0..std::mem::size_of::<PushConstants>() as u32,
                }],
            },
        );
        let vs_module = roe_graphics::ShaderModule::new(
            &instance,
            roe_graphics::include_spirv!("shaders/gen/spirv/shape2.vert.spv"),
        );
        let fs_module = roe_graphics::ShaderModule::new(
            &instance,
            roe_graphics::include_spirv!("shaders/gen/spirv/shape2.frag.spv"),
        );
        let pipeline = roe_graphics::RenderPipeline::new(
            &instance,
            &roe_graphics::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex_stage: roe_graphics::ProgrammableStageDescriptor {
                    module: &vs_module,
                    entry_point: "main",
                },
                fragment_stage: Some(roe_graphics::ProgrammableStageDescriptor {
                    module: &fs_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(roe_graphics::RasterizationStateDescriptor {
                    front_face: roe_graphics::FrontFace::Ccw,
                    cull_mode: roe_graphics::CullMode::Back,
                    ..Default::default()
                }),
                primitive_topology: roe_graphics::PrimitiveTopology::TriangleList,
                color_states: &[roe_graphics::ColorStateDescriptor {
                    format: roe_graphics::TextureFormat::from(desc.color_buffer_format),
                    color_blend: desc.color_blend.clone(),
                    alpha_blend: desc.alpha_blend.clone(),
                    write_mask: desc.write_mask,
                }],
                depth_stencil_state: None,
                vertex_state: roe_graphics::VertexStateDescriptor {
                    index_format: roe_graphics::IndexFormat::Uint16,
                    vertex_buffers: &[roe_graphics::VertexBufferDescriptor {
                        stride: std::mem::size_of::<Vertex>() as roe_graphics::BufferAddress,
                        step_mode: roe_graphics::InputStepMode::Vertex,
                        attributes: &[roe_graphics::VertexAttributeDescriptor {
                            format: roe_graphics::VertexFormat::Float2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }],
                },
                sample_count: desc.sample_count,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            },
        );
        Self {
            pipeline,
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
    fn draw_shape2(
        &mut self,
        pipeline: &'a RenderPipeline,
        mesh: &'a Mesh,
        push_constants: &'a PushConstants,
        index_range: MeshIndexRange,
    );

    fn draw_shape2_array<MeshIt, PcIt, RangeIt>(
        &mut self,
        pipeline: &'a RenderPipeline,
        draw_commands: MeshIt,
    ) where
        MeshIt: IntoIterator<Item = (&'a Mesh, PcIt)>,
        PcIt: IntoIterator<Item = (&'a PushConstants, RangeIt)>,
        RangeIt: IntoIterator<Item = roe_graphics::MeshIndexRange>;
}

impl<'a> Renderer<'a> for roe_graphics::RenderPass<'a> {
    fn draw_shape2(
        &mut self,
        pipeline: &'a RenderPipeline,
        mesh: &'a Mesh,
        push_constants: &'a PushConstants,
        index_range: MeshIndexRange,
    ) {
        self.set_pipeline(&pipeline.pipeline);
        self.set_index_buffer(mesh.index_buffer().slice(..));
        self.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
        self.set_push_constants(roe_graphics::ShaderStage::VERTEX, 0, push_constants.as_slice());
        self.draw_indexed(index_range, 0, 0..1);
    }

    fn draw_shape2_array<MeshIt, PcIt, RangeIt>(
        &mut self,
        pipeline: &'a RenderPipeline,
        draw_commands: MeshIt,
    ) where
        MeshIt: IntoIterator<Item = (&'a Mesh, PcIt)>,
        PcIt: IntoIterator<Item = (&'a PushConstants, RangeIt)>,
        RangeIt: IntoIterator<Item = roe_graphics::MeshIndexRange>,
    {
        self.set_pipeline(&pipeline.pipeline);
        for (mesh, pcs) in draw_commands.into_iter() {
            self.set_index_buffer(mesh.index_buffer().slice(..));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn creation() {
        let instance = roe_graphics::Instance::new(&roe_graphics::InstanceDescriptor::default()).unwrap();
        let _pipeline = RenderPipeline::new(&instance, &RenderPipelineDescriptor::default());
    }
}
