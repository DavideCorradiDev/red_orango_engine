// TODO: re-rename renamed types.
pub use wgpu::{
    include_spirv, util::BufferInitDescriptor, AdapterInfo, AddressMode,
    Backends as Backend, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendFactor, BlendOperation, BufferAddress,
    BufferDescriptor, BufferSlice, BufferUsages as BufferUsage, CommandBuffer,
    CommandEncoderDescriptor, CompareFunction, Extent3d, Features, FilterMode, FrontFace,
    ImageCopyBuffer as BufferCopyView, ImageCopyTexture as TextureCopyView,
    ImageDataLayout as TextureDataLayout, IndexFormat, Limits, LoadOp, Maintain, MapMode,
    Operations, Origin3d, PipelineLayoutDescriptor, PowerPreference, PresentMode,
    PrimitiveTopology, PushConstantRange, RenderBundleEncoderDescriptor, RenderPass,
    RenderPassDescriptor, RenderPipelineDescriptor, SamplerDescriptor, ShaderModuleDescriptor,
    ShaderSource, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexFormat,
};

pub use wgpu::{
    BlendDescriptor, ColorStateDescriptor, ColorWrite, CullMode, DepthStencilStateDescriptor,
    InputStepMode, ProgrammableStageDescriptor, RasterizationStateDescriptor,
    RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor, ShaderStage,
    StencilStateDescriptor, SwapChainDescriptor, SwapChainError, SwapChainFrame, SwapChainTexture,
    TextureComponentType, TextureUsage, VertexAttributeDescriptor, VertexBufferDescriptor,
    VertexStateDescriptor,
};

mod size;
pub use size::*;

mod color;
pub use color::*;

mod instance;
pub use instance::*;

mod canvas;
pub use canvas::*;

mod canvas_window;
pub use canvas_window::*;

mod canvas_texture;
pub use canvas_texture::*;

mod command_sequence;
pub use command_sequence::*;

mod mesh;
pub use mesh::*;
