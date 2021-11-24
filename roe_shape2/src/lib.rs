use std::{default::Default, iter::IntoIterator};

use num_traits::Zero;

use roe_graphics as gfx;

use roe_math::{conversion::ToHomogeneousMatrix3, geometry2, geometry3};

#[repr(C, packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
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

pub type MeshIndexRange = gfx::MeshIndexRange;
pub type MeshIndex = gfx::MeshIndex;
pub type Mesh = gfx::IndexedMesh<Vertex>;

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
    sample_count: gfx::SampleCount,
    color_buffer_format: gfx::CanvasColorBufferFormat,
}

impl RenderPipeline {
    pub fn new(instance: &gfx::Instance, desc: &RenderPipelineDescriptor) -> Self {
        let pipeline_layout = gfx::PipelineLayout::new(
            &instance,
            &gfx::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[gfx::PushConstantRange {
                    stages: gfx::ShaderStage::VERTEX,
                    range: 0..std::mem::size_of::<PushConstants>() as u32,
                }],
            },
        );
        let vs_module = gfx::ShaderModule::new(
            instance,
            &gfx::include_spirv!("shaders/gen/spirv/shape2.vert.spv"),
        );
        let fs_module = gfx::ShaderModule::new(
            instance,
            &gfx::include_spirv!("shaders/gen/spirv/shape2.frag.spv"),
        );
        let pipeline = gfx::RenderPipeline::new(
            &instance,
            &gfx::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: gfx::VertexState {
                    module: &vs_module,
                    entry_point: "main",
                    buffers: &[gfx::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as gfx::BufferAddress,
                        step_mode: gfx::VertexStepMode::Vertex,
                        attributes: &[gfx::VertexAttribute {
                            format: gfx::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
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
        RangeIt: IntoIterator<Item = gfx::MeshIndexRange>;
}

impl<'a> Renderer<'a> for gfx::RenderPass<'a> {
    fn draw_shape2(
        &mut self,
        pipeline: &'a RenderPipeline,
        mesh: &'a Mesh,
        push_constants: &'a PushConstants,
        index_range: MeshIndexRange,
    ) {
        self.set_pipeline(&pipeline.pipeline);
        self.set_index_buffer(mesh.index_buffer().slice(..), gfx::IndexFormat::Uint16);
        self.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
        self.set_push_constants(
            gfx::ShaderStage::VERTEX,
            0,
            gfx::utility::as_slice(push_constants),
        );
        self.draw_indexed(index_range, 0, 0..1);
    }

    fn draw_shape2_array<MeshIt, PcIt, RangeIt>(
        &mut self,
        pipeline: &'a RenderPipeline,
        draw_commands: MeshIt,
    ) where
        MeshIt: IntoIterator<Item = (&'a Mesh, PcIt)>,
        PcIt: IntoIterator<Item = (&'a PushConstants, RangeIt)>,
        RangeIt: IntoIterator<Item = gfx::MeshIndexRange>,
    {
        self.set_pipeline(&pipeline.pipeline);
        for (mesh, pcs) in draw_commands.into_iter() {
            self.set_index_buffer(mesh.index_buffer().slice(..), gfx::IndexFormat::Uint16);
            self.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
            for (pc, ranges) in pcs.into_iter() {
                self.set_push_constants(gfx::ShaderStage::VERTEX, 0, gfx::utility::as_slice(pc));
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
    fn draw_shape2() {
        let instance = gfx::Instance::new(&gfx::InstanceDescriptor::default()).unwrap();
        let mut canvas = gfx::CanvasTexture::new(
            &instance,
            &gfx::CanvasTextureDescriptor {
                size: gfx::CanvasSize::new(100, 100),
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

        let mesh = Mesh::new(
            &instance,
            &[
                Vertex::new([-50., 50.]),
                Vertex::new([50., 50.]),
                Vertex::new([0., -50.]),
            ],
            &[0, 1, 2],
        );

        let projection_transform =
            geo::OrthographicProjection::new(0., 100., 100., 0.).to_projective();

        let constants_1 = PushConstants::new(
            &convert(
                projection_transform
                    * geo::Similarity::<f32>::from_parts(
                        geo::Translation::new(50., 60.),
                        geo::UnitComplex::new(std::f32::consts::PI * 0.5),
                        0.5,
                    ),
            ),
            gfx::ColorF32::CYAN,
        );

        let constants_2 = PushConstants::new(
            &convert(
                projection_transform
                    * geo::Similarity::<f32>::from_parts(
                        geo::Translation::new(70., 30.),
                        geo::UnitComplex::new(0.),
                        0.25,
                    ),
            ),
            gfx::ColorF32::RED,
        );

        let constants_3 = PushConstants::new(
            &convert(
                projection_transform
                    * geo::Similarity::<f32>::from_parts(
                        geo::Translation::new(20., 80.),
                        geo::UnitComplex::new(std::f32::consts::PI),
                        0.15,
                    ),
            ),
            gfx::ColorF32::YELLOW,
        );

        {
            let frame = canvas.current_frame().unwrap().unwrap();
            let mut cmd_sequence = gfx::CommandSequence::new(&instance);
            {
                let mut rpass = cmd_sequence.begin_render_pass(
                    &frame,
                    &pipeline.render_pass_requirements(),
                    &gfx::RenderPassOperations::default(),
                );
                rpass.draw_shape2(&pipeline, &mesh, &constants_1, 0..mesh.index_count());
                rpass.draw_shape2_array(
                    &pipeline,
                    [(
                        &mesh,
                        [
                            (&constants_2, [0..mesh.index_count()]),
                            (&constants_3, [0..mesh.index_count()]),
                        ],
                    )],
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
