use std::{default::Default, iter};

use super::{
    CanvasColorBufferFormat, CanvasDepthStencilBufferFormat, CanvasFrame, ColorOperations,
    CommandEncoder, CommandEncoderDescriptor, DepthOperations, Instance, Operations, RenderPass,
    RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, SampleCount, StencilOperations,
};

#[derive(Debug, PartialEq, Clone)]
pub struct RenderPassRequirements {
    pub sample_count: SampleCount,
    pub color_buffer_formats: Vec<CanvasColorBufferFormat>,
    pub depth_stencil_buffer_format: Option<CanvasDepthStencilBufferFormat>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RenderPassOperations {
    pub color_operations: Vec<ColorOperations>,
    pub depth_operations: Option<DepthOperations>,
    pub stencil_operations: Option<StencilOperations>,
}

impl Default for RenderPassOperations {
    fn default() -> Self {
        RenderPassOperations {
            color_operations: Vec::new(),
            depth_operations: None,
            stencil_operations: None,
        }
    }
}

#[derive(Debug)]
pub struct CommandSequence {
    encoder: CommandEncoder,
}

impl CommandSequence {
    pub fn new(instance: &Instance) -> Self {
        let encoder = CommandEncoder::new(&instance, &CommandEncoderDescriptor::default());
        Self { encoder }
    }

    pub fn begin_render_pass<'a>(
        &'a mut self,
        canvas_frame: &'a CanvasFrame,
        requirements: &RenderPassRequirements,
        operations: &RenderPassOperations,
    ) -> RenderPass<'a> {
        // Define color attachments.
        let has_swap_chain = canvas_frame.swap_chain().is_some();
        let required_color_buffer_count = requirements.color_buffer_formats.len();
        let mut color_attachments = Vec::with_capacity(required_color_buffer_count);
        for i in 0..required_color_buffer_count {
            let required_format = requirements.color_buffer_formats[i];
            let ops = match operations.color_operations.get(i) {
                Some(v) => *v,
                None => Operations::default(),
            };
            if i == 0 && has_swap_chain {
                let swap_chain = canvas_frame.swap_chain().unwrap();
                assert!(
                    required_format == swap_chain.format()
                        && requirements.sample_count == swap_chain.sample_count(),
                    "Incompatible swap chain"
                );
                color_attachments.push(RenderPassColorAttachmentDescriptor {
                    attachment: swap_chain.attachment(),
                    resolve_target: swap_chain.resolve_target(),
                    ops,
                });
            } else {
                let buffer_index = i - if has_swap_chain { 1 } else { 0 };
                let color_buffer = canvas_frame
                    .color_buffers()
                    .get(buffer_index)
                    .expect("Not enough color buffers");
                assert!(
                    required_format == color_buffer.format()
                        && requirements.sample_count == color_buffer.sample_count(),
                    "Incompatible color buffer"
                );
                color_attachments.push(RenderPassColorAttachmentDescriptor {
                    attachment: color_buffer.attachment(),
                    resolve_target: color_buffer.resolve_target(),
                    ops,
                })
            }
        }

        // Define depth stencil attachments.
        let depth_stencil_attachment = match requirements.depth_stencil_buffer_format {
            Some(required_format) => match canvas_frame.depth_stencil_buffer() {
                Some(ds_buffer) => {
                    assert!(
                        required_format == ds_buffer.format()
                            && requirements.sample_count == ds_buffer.sample_count(),
                        "Incompatible depth stencil buffer"
                    );
                    Some(RenderPassDepthStencilAttachmentDescriptor {
                        attachment: ds_buffer.attachment(),
                        depth_ops: operations.depth_operations,
                        stencil_ops: operations.stencil_operations,
                    })
                }
                None => panic!("Unavailable depth stencil buffer"),
            },
            None => None,
        };

        // Begin the render pass.
        let render_pass_desc = RenderPassDescriptor {
            label: Some("RoeRenderPass"),
            color_attachments: color_attachments.as_slice(),
            depth_stencil_attachment,
        };
        self.encoder.begin_render_pass(&render_pass_desc)
    }

    pub fn submit(self, instance: &Instance) {
        instance.submit(iter::once(self.encoder.finish()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        CanvasBuffer, CanvasBufferColorBufferDescriptor, CanvasBufferDescriptor,
        CanvasColorBufferUsage, CanvasSize, InstanceDescriptor,
    };

    #[test]
    #[serial_test::serial]
    fn creation() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let _cmd_seq = CommandSequence::new(&instance);
    }

    #[test]
    #[serial_test::serial]
    fn render_pass() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let mut cmd_seq = CommandSequence::new(&instance);
        let mut buffer = CanvasBuffer::new(
            &instance,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                swap_chain_descriptor: None,
                color_buffer_descriptors: vec![CanvasBufferColorBufferDescriptor {
                    format: CanvasColorBufferFormat::default(),
                    usage: CanvasColorBufferUsage::empty(),
                }],
                depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
            },
        );

        {
            let frame = buffer.current_frame().unwrap();
            let _rpass = cmd_seq.begin_render_pass(
                &frame,
                &RenderPassRequirements {
                    sample_count: 2,
                    color_buffer_formats: vec![CanvasColorBufferFormat::default()],
                    depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
                },
                &RenderPassOperations::default(),
            );
        }

        {
            let frame = buffer.current_frame().unwrap();
            let _rpass = cmd_seq.begin_render_pass(
                &frame,
                &RenderPassRequirements {
                    sample_count: 2,
                    color_buffer_formats: vec![CanvasColorBufferFormat::default()],
                    depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
                },
                &RenderPassOperations::default(),
            );
        }

        cmd_seq.submit(&instance);
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "Incompatible color buffer")]
    fn begin_render_pass_error_incompatible_sample_count() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let mut cmd_seq = CommandSequence::new(&instance);
        let mut buffer = CanvasBuffer::new(
            &instance,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 1,
                swap_chain_descriptor: None,
                color_buffer_descriptors: vec![CanvasBufferColorBufferDescriptor {
                    format: CanvasColorBufferFormat::default(),
                    usage: CanvasColorBufferUsage::empty(),
                }],
                depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
            },
        );

        {
            let frame = buffer.current_frame().unwrap();
            let _rpass = cmd_seq.begin_render_pass(
                &frame,
                &RenderPassRequirements {
                    sample_count: 2,
                    color_buffer_formats: vec![CanvasColorBufferFormat::default()],
                    depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
                },
                &RenderPassOperations::default(),
            );
        }
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "Not enough color buffers")]
    fn begin_render_pass_error_not_enough_color_buffers() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let mut cmd_seq = CommandSequence::new(&instance);
        let mut buffer = CanvasBuffer::new(
            &instance,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                swap_chain_descriptor: None,
                color_buffer_descriptors: Vec::new(),
                depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
            },
        );

        let frame = buffer.current_frame().unwrap();
        let _rpass = cmd_seq.begin_render_pass(
            &frame,
            &RenderPassRequirements {
                sample_count: 2,
                color_buffer_formats: vec![CanvasColorBufferFormat::default()],
                depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
            },
            &RenderPassOperations::default(),
        );
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "Incompatible color buffer")]
    fn begin_render_pass_error_wrong_color_buffer_format() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let mut cmd_seq = CommandSequence::new(&instance);
        let mut buffer = CanvasBuffer::new(
            &instance,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                swap_chain_descriptor: None,
                color_buffer_descriptors: vec![CanvasBufferColorBufferDescriptor {
                    format: CanvasColorBufferFormat::Bgra8Unorm,
                    usage: CanvasColorBufferUsage::empty(),
                }],
                depth_stencil_buffer_format: None,
            },
        );

        let frame = buffer.current_frame().unwrap();
        let _rpass = cmd_seq.begin_render_pass(
            &frame,
            &RenderPassRequirements {
                sample_count: 2,
                color_buffer_formats: vec![CanvasColorBufferFormat::Bgra8UnormSrgb],
                depth_stencil_buffer_format: None,
            },
            &RenderPassOperations::default(),
        );
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "Unavailable depth stencil buffer")]
    fn begin_render_pass_error_no_depth_stencil_buffer() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let mut cmd_seq = CommandSequence::new(&instance);
        let mut buffer = CanvasBuffer::new(
            &instance,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                swap_chain_descriptor: None,
                color_buffer_descriptors: vec![CanvasBufferColorBufferDescriptor {
                    format: CanvasColorBufferFormat::default(),
                    usage: CanvasColorBufferUsage::empty(),
                }],
                depth_stencil_buffer_format: None,
            },
        );

        let frame = buffer.current_frame().unwrap();
        let _rpass = cmd_seq.begin_render_pass(
            &frame,
            &RenderPassRequirements {
                sample_count: 2,
                color_buffer_formats: vec![CanvasColorBufferFormat::default()],
                depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
            },
            &RenderPassOperations::default(),
        );
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "Incompatible depth stencil buffer")]
    fn begin_render_pass_error_wrong_depth_stencil_buffer_format() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let mut cmd_seq = CommandSequence::new(&instance);
        let mut buffer = CanvasBuffer::new(
            &instance,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                swap_chain_descriptor: None,
                color_buffer_descriptors: Vec::new(),
                depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth24Plus),
            },
        );

        let frame = buffer.current_frame().unwrap();
        let _rpass = cmd_seq.begin_render_pass(
            &frame,
            &RenderPassRequirements {
                sample_count: 2,
                color_buffer_formats: Vec::new(),
                depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
            },
            &RenderPassOperations::default(),
        );
    }
}
