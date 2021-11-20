use super::{
    Extent3d, Instance, PresentMode, SampleCount, Size, Surface, SurfaceError, SurfaceTexture,
    Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage, TextureView,
    TextureViewDescriptor, SurfaceConfiguration
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CanvasColorBufferFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bgra8Unorm,
    Bgra8UnormSrgb,
}

impl Default for CanvasColorBufferFormat {
    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        Self::Bgra8Unorm
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn default() -> Self {
        Self::Bgra8UnormSrgb
    }
}

impl From<CanvasColorBufferFormat> for TextureFormat {
    fn from(f: CanvasColorBufferFormat) -> Self {
        match f {
            CanvasColorBufferFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
            CanvasColorBufferFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
            CanvasColorBufferFormat::Bgra8Unorm => TextureFormat::Bgra8Unorm,
            CanvasColorBufferFormat::Bgra8UnormSrgb => TextureFormat::Bgra8UnormSrgb,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CanvasDepthStencilBufferFormat {
    Depth32Float,
    Depth24Plus,
    Depth24PlusStencil8,
}

impl From<CanvasDepthStencilBufferFormat> for TextureFormat {
    fn from(f: CanvasDepthStencilBufferFormat) -> Self {
        match f {
            CanvasDepthStencilBufferFormat::Depth32Float => TextureFormat::Depth32Float,
            CanvasDepthStencilBufferFormat::Depth24Plus => TextureFormat::Depth24Plus,
            CanvasDepthStencilBufferFormat::Depth24PlusStencil8 => {
                TextureFormat::Depth24PlusStencil8
            }
        }
    }
}

pub type CanvasSize = Size<u32>;

#[derive(Debug)]
pub struct CanvasSwapChainRef<'a> {
    sample_count: SampleCount,
    format: CanvasColorBufferFormat,
    multisampled_buffer: Option<&'a TextureView>,
    // TODO: change to reference?
    frame: TextureView,
}

impl<'a> CanvasSwapChainRef<'a> {
    pub fn attachment(&self) -> &TextureView {
        match self.multisampled_buffer {
            Some(v) => &v,
            None => &self.frame,
        }
    }

    pub fn resolve_target(&self) -> Option<&TextureView> {
        match self.multisampled_buffer {
            Some(_) => Some(&self.frame),
            None => None,
        }
    }

    pub fn sample_count(&self) -> SampleCount {
        self.sample_count
    }

    pub fn format(&self) -> CanvasColorBufferFormat {
        self.format
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CanvasSwapChainDescriptor {
    pub size: CanvasSize,
    pub sample_count: SampleCount,
    pub format: CanvasColorBufferFormat,
}

#[derive(Debug)]
pub struct CanvasSwapChain {
    size: CanvasSize,
    sample_count: SampleCount,
    format: CanvasColorBufferFormat,
    multisampled_buffer: Option<TextureView>,
    surface: Surface,
}

impl CanvasSwapChain {
    pub fn new(instance: &Instance, mut surface: Surface, desc: &CanvasSwapChainDescriptor) -> Self {
        let usage = TextureUsage::RENDER_ATTACHMENT;
        let texture_format = TextureFormat::from(desc.format);
        let width = desc.size.width();
        let height = desc.size.height();
        surface.configure(instance, &SurfaceConfiguration {
                usage,
                format: texture_format,
                width,
                height,
                present_mode: PresentMode::Mailbox,
        });
        let multisampled_buffer = if desc.sample_count > 1 {
            let multisampling_buffer_texture = Texture::new(
                instance,
                &TextureDescriptor {
                    size: Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: desc.sample_count,
                    dimension: TextureDimension::D2,
                    format: texture_format,
                    usage,
                    label: None,
                },
            );
            Some(multisampling_buffer_texture.create_view(&TextureViewDescriptor::default()))
        } else {
            None
        };
        Self {
            size: desc.size,
            sample_count: desc.sample_count,
            format: desc.format,
            multisampled_buffer,
            surface,
        }
    }

    pub fn size(&self) -> &CanvasSize {
        &self.size
    }

    pub fn sample_count(&self) -> SampleCount {
        self.sample_count
    }

    pub fn format(&self) -> CanvasColorBufferFormat {
        self.format
    }

    pub fn reference(&mut self) -> Result<CanvasSwapChainRef, SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        // TODO: view descriptor should be populated...
        let frame = surface_texture.texture.create_view(&TextureViewDescriptor::default());
        let multisampled_buffer = match self.multisampled_buffer {
            Some(ref v) => Some(v),
            None => None,
        };
        Ok(CanvasSwapChainRef {
            sample_count: self.sample_count,
            format: self.format,
            multisampled_buffer,
            frame,
        })
    }

    pub fn configure(&mut self, instance: &Instance, desc: &CanvasSwapChainDescriptor) {
        // TODO: remove repetition.
        let usage = TextureUsage::RENDER_ATTACHMENT;
        let texture_format = TextureFormat::from(desc.format);
        let width = desc.size.width();
        let height = desc.size.height();
        surface.configure(instance, &SurfaceConfiguration {
                usage,
                format: texture_format,
                width,
                height,
                present_mode: PresentMode::Mailbox,
        });
        let multisampled_buffer = if desc.sample_count > 1 {
            let multisampling_buffer_texture = Texture::new(
                instance,
                &TextureDescriptor {
                    size: Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: desc.sample_count,
                    dimension: TextureDimension::D2,
                    format: texture_format,
                    usage,
                    label: None,
                },
            );
            Some(multisampling_buffer_texture.create_view(&TextureViewDescriptor::default()))
        } else {
            None
        };

        let current_size = self.inner_size();
        let current_size = CanvasSize::new(current_size.width, current_size.height);
        if *self.canvas_size() != current_size {
            self.canvas_buffer = CanvasBuffer::new(
                instance,
                &CanvasBufferDescriptor {
                    size: current_size,
                    sample_count: self.sample_count(),
                    swap_chain_descriptor: Some(CanvasBufferSwapChainDescriptor {
                        surface: &self.surface,
                        format: self.color_buffer_format(),
                    }),
                    color_buffer_descriptors: Vec::new(),
                    depth_stencil_buffer_format: self.depth_stencil_buffer_format(),
                },
            );
        }
    }
}

#[derive(Debug)]
pub struct CanvasColorBufferRef<'a> {
    sample_count: SampleCount,
    format: CanvasColorBufferFormat,
    multisampled_buffer: Option<&'a TextureView>,
    main_buffer: &'a TextureView,
}

impl<'a> CanvasColorBufferRef<'a> {
    pub fn attachment(&self) -> &TextureView {
        match self.multisampled_buffer {
            Some(v) => v,
            None => self.main_buffer,
        }
    }

    pub fn resolve_target(&self) -> Option<&TextureView> {
        match self.multisampled_buffer {
            Some(_) => Some(self.main_buffer),
            None => None,
        }
    }

    pub fn sample_count(&self) -> SampleCount {
        self.sample_count
    }

    pub fn format(&self) -> CanvasColorBufferFormat {
        self.format
    }
}

bitflags::bitflags! {
    pub struct CanvasColorBufferUsage : u32 {
        const COPY_SRC = TextureUsage::COPY_SRC.bits();
        const COPY_DST = TextureUsage::COPY_DST.bits();
        const TEXTURE_BINDING = TextureUsage::TEXTURE_BINDING.bits();
    }
}

impl From<CanvasColorBufferUsage> for TextureUsage {
    fn from(usage: CanvasColorBufferUsage) -> Self {
        TextureUsage::from_bits(usage.bits()).unwrap()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CanvasColorBufferDescriptor {
    pub size: CanvasSize,
    pub sample_count: SampleCount,
    pub format: CanvasColorBufferFormat,
    pub usage: CanvasColorBufferUsage,
}

#[derive(Debug)]
pub struct CanvasColorBuffer {
    size: CanvasSize,
    sample_count: SampleCount,
    format: CanvasColorBufferFormat,
    multisampled_buffer: Option<TextureView>,
    main_buffer_view: TextureView,
    main_buffer_texture: Texture,
}

impl CanvasColorBuffer {
    pub fn new(instance: &Instance, desc: &CanvasColorBufferDescriptor) -> Self {
        let mut tex_desc = TextureDescriptor {
            size: Extent3d {
                width: desc.size.width(),
                height: desc.size.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::from(desc.format),
            usage: TextureUsage::from(desc.usage) | TextureUsage::RENDER_ATTACHMENT,
            label: None,
        };

        let main_buffer_texture = Texture::new(instance, &tex_desc);
        let main_buffer_view = main_buffer_texture.create_view(&TextureViewDescriptor::default());

        let multisampled_buffer = if desc.sample_count > 1 {
            tex_desc.sample_count = desc.sample_count;
            Some(Texture::new(instance, &tex_desc).create_view(&TextureViewDescriptor::default()))
        } else {
            None
        };

        Self {
            size: desc.size,
            sample_count: desc.sample_count,
            format: desc.format,
            multisampled_buffer,
            main_buffer_view,
            main_buffer_texture,
        }
    }

    pub fn size(&self) -> &CanvasSize {
        &self.size
    }

    pub fn sample_count(&self) -> SampleCount {
        self.sample_count
    }

    pub fn format(&self) -> CanvasColorBufferFormat {
        self.format
    }

    pub fn texture_view(&self) -> &TextureView {
        &self.main_buffer_view
    }

    pub fn texture(&self) -> &Texture {
        &self.main_buffer_texture
    }

    pub fn reference(&self) -> CanvasColorBufferRef {
        let multisampled_buffer = match self.multisampled_buffer {
            Some(ref v) => Some(v),
            None => None,
        };
        CanvasColorBufferRef {
            sample_count: self.sample_count,
            format: self.format,
            multisampled_buffer,
            main_buffer: &self.main_buffer_view,
        }
    }
}

#[derive(Debug)]
pub struct CanvasDepthStencilBufferRef<'a> {
    sample_count: SampleCount,
    format: CanvasDepthStencilBufferFormat,
    buffer: &'a TextureView,
}

impl<'a> CanvasDepthStencilBufferRef<'a> {
    pub fn attachment(&self) -> &TextureView {
        self.buffer
    }

    pub fn sample_count(&self) -> SampleCount {
        self.sample_count
    }

    pub fn format(&self) -> CanvasDepthStencilBufferFormat {
        self.format
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CanvasDepthStencilBufferDescriptor {
    pub size: CanvasSize,
    pub sample_count: SampleCount,
    pub format: CanvasDepthStencilBufferFormat,
}

#[derive(Debug)]
pub struct CanvasDepthStencilBuffer {
    size: CanvasSize,
    sample_count: SampleCount,
    format: CanvasDepthStencilBufferFormat,
    buffer_view: TextureView,
    buffer_texture: Texture,
}

impl CanvasDepthStencilBuffer {
    pub fn new(instance: &Instance, desc: &CanvasDepthStencilBufferDescriptor) -> Self {
        let buffer_texture = Texture::new(
            instance,
            &TextureDescriptor {
                size: Extent3d {
                    width: desc.size.width(),
                    height: desc.size.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: desc.sample_count,
                dimension: TextureDimension::D2,
                format: TextureFormat::from(desc.format),
                usage: TextureUsage::RENDER_ATTACHMENT,
                label: None,
            },
        );
        let buffer_view = buffer_texture.create_view(&TextureViewDescriptor::default());
        Self {
            size: desc.size,
            sample_count: desc.sample_count,
            format: desc.format,
            buffer_view,
            buffer_texture,
        }
    }

    pub fn size(&self) -> &CanvasSize {
        &self.size
    }

    pub fn sample_count(&self) -> SampleCount {
        self.sample_count
    }

    pub fn format(&self) -> CanvasDepthStencilBufferFormat {
        self.format
    }

    pub fn texture_view(&self) -> &TextureView {
        &self.buffer_view
    }

    pub fn texture(&self) -> &Texture {
        &self.buffer_texture
    }

    pub fn reference(&self) -> CanvasDepthStencilBufferRef {
        CanvasDepthStencilBufferRef {
            sample_count: self.sample_count,
            format: self.format,
            buffer: &self.buffer_view,
        }
    }
}

#[derive(Debug)]
pub struct CanvasFrame<'a> {
    swap_chain: Option<CanvasSwapChainRef<'a>>,
    color_buffers: Vec<CanvasColorBufferRef<'a>>,
    depth_stencil_buffer: Option<CanvasDepthStencilBufferRef<'a>>,
}

impl<'a> CanvasFrame<'a> {
    pub fn swap_chain(&self) -> Option<&CanvasSwapChainRef<'a>> {
        self.swap_chain.as_ref()
    }

    pub fn color_buffers(&self) -> &Vec<CanvasColorBufferRef<'a>> {
        &self.color_buffers
    }

    pub fn depth_stencil_buffer(&self) -> Option<&CanvasDepthStencilBufferRef<'a>> {
        self.depth_stencil_buffer.as_ref()
    }
}

#[derive(Debug)]
pub struct CanvasBufferSwapChainDescriptor {
    pub surface: Surface,
    pub format: CanvasColorBufferFormat,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CanvasBufferColorBufferDescriptor {
    pub format: CanvasColorBufferFormat,
    pub usage: CanvasColorBufferUsage,
}

impl Default for CanvasBufferColorBufferDescriptor {
    fn default() -> Self {
        Self {
            format: CanvasColorBufferFormat::default(),
            usage: CanvasColorBufferUsage::empty(),
        }
    }
}

#[derive(Debug)]
pub struct CanvasBufferDescriptor {
    pub size: CanvasSize,
    pub sample_count: SampleCount,
    pub swap_chain_descriptor: Option<CanvasBufferSwapChainDescriptor>,
    pub color_buffer_descriptors: Vec<CanvasBufferColorBufferDescriptor>,
    pub depth_stencil_buffer_format: Option<CanvasDepthStencilBufferFormat>,
}

#[derive(Debug)]
pub struct CanvasBuffer {
    size: CanvasSize,
    sample_count: SampleCount,
    swap_chain: Option<CanvasSwapChain>,
    color_buffers: Vec<CanvasColorBuffer>,
    depth_stencil_buffer: Option<CanvasDepthStencilBuffer>,
}

impl CanvasBuffer {
    pub fn new(instance: &Instance, desc: &CanvasBufferDescriptor) -> Self {
        let swap_chain = match &desc.swap_chain_descriptor {
            Some(sc_desc) => Some(CanvasSwapChain::new(
                instance,
                sc_desc.surface,
                &CanvasSwapChainDescriptor {
                    size: desc.size,
                    sample_count: desc.sample_count,
                    format: sc_desc.format,
                },
            )),
            None => None,
        };

        let mut color_buffers = Vec::with_capacity(desc.color_buffer_descriptors.len());
        for cbd in desc.color_buffer_descriptors.iter() {
            color_buffers.push(CanvasColorBuffer::new(
                instance,
                &CanvasColorBufferDescriptor {
                    size: desc.size,
                    sample_count: desc.sample_count,
                    format: cbd.format,
                    usage: cbd.usage,
                },
            ));
        }

        let depth_stencil_buffer = match &desc.depth_stencil_buffer_format {
            Some(format) => Some(CanvasDepthStencilBuffer::new(
                instance,
                &CanvasDepthStencilBufferDescriptor {
                    size: desc.size,
                    sample_count: desc.sample_count,
                    format: *format,
                },
            )),
            None => None,
        };

        assert!(
            swap_chain.is_some() || !color_buffers.is_empty() || depth_stencil_buffer.is_some(),
            "No buffer defined for a canvas buffer"
        );

        Self {
            size: desc.size,
            sample_count: desc.sample_count,
            swap_chain,
            color_buffers,
            depth_stencil_buffer,
        }
    }

    pub fn size(&self) -> &CanvasSize {
        &self.size
    }

    pub fn sample_count(&self) -> SampleCount {
        self.sample_count
    }

    pub fn swap_chain(&self) -> Option<&CanvasSwapChain> {
        self.swap_chain.as_ref()
    }

    pub fn color_buffers(&self) -> &Vec<CanvasColorBuffer> {
        &self.color_buffers
    }

    pub fn depth_stencil_buffer(&self) -> Option<&CanvasDepthStencilBuffer> {
        self.depth_stencil_buffer.as_ref()
    }

    pub fn current_frame(&mut self) -> Result<CanvasFrame, SurfaceError> {
        let swap_chain = match &mut self.swap_chain {
            Some(swap_chain) => Some(swap_chain.reference()?),
            None => None,
        };

        let mut color_buffers = Vec::with_capacity(self.color_buffers.len());
        for color_buffer in self.color_buffers.iter() {
            color_buffers.push(color_buffer.reference());
        }

        let depth_stencil_buffer = match &self.depth_stencil_buffer {
            Some(depth_stencil_buffer) => Some(depth_stencil_buffer.reference()),
            None => None,
        };

        Ok(CanvasFrame {
            swap_chain,
            color_buffers,
            depth_stencil_buffer,
        })
    }
}

pub trait Canvas {
    fn current_frame(&mut self) -> Result<CanvasFrame, SurfaceError>;
    fn canvas_size(&self) -> &CanvasSize;
    fn sample_count(&self) -> SampleCount;
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    use roe_app::{
        event::{EventLoop, EventLoopAnyThread},
        window::WindowBuilder,
    };

    use crate::InstanceDescriptor;

    #[test]
    #[serial_test::serial]
    fn canvas_swap_chain() {
        let event_loop = EventLoop::<()>::new_any_thread();
        let window = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        let (instance, surface) = unsafe {
            Instance::new_with_compatible_window(&InstanceDescriptor::default(), &window).unwrap()
        };

        let mut swap_chain = CanvasSwapChain::new(
            &instance,
            surface,
            &CanvasSwapChainDescriptor {
                sample_count: 2,
                format: CanvasColorBufferFormat::Bgra8Unorm,
                size: CanvasSize::new(12, 20),
            },
        );

        expect_that!(&swap_chain.sample_count(), eq(2));
        expect_that!(
            &swap_chain.format(),
            eq(CanvasColorBufferFormat::Bgra8Unorm)
        );
        expect_that!(swap_chain.size(), eq(CanvasSize::new(12, 20)));

        let reference = swap_chain.reference().unwrap();
        expect_that!(&reference.sample_count(), eq(2));
        expect_that!(&reference.format(), eq(CanvasColorBufferFormat::Bgra8Unorm));
        expect_that!(reference.resolve_target().is_some());
    }

    #[test]
    #[serial_test::serial]
    fn canvas_color_buffer() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let buffer = CanvasColorBuffer::new(
            &instance,
            &CanvasColorBufferDescriptor {
                sample_count: 2,
                format: CanvasColorBufferFormat::Bgra8Unorm,
                size: CanvasSize::new(12, 20),
                usage: CanvasColorBufferUsage::empty(),
            },
        );

        expect_that!(&buffer.sample_count(), eq(2));
        expect_that!(&buffer.format(), eq(CanvasColorBufferFormat::Bgra8Unorm));
        expect_that!(buffer.size(), eq(CanvasSize::new(12, 20)));

        let reference = buffer.reference();
        expect_that!(&reference.sample_count(), eq(2));
        expect_that!(&reference.format(), eq(CanvasColorBufferFormat::Bgra8Unorm));
        expect_that!(reference.resolve_target().is_some());
    }

    #[test]
    #[serial_test::serial]
    fn canvas_depth_stencil_buffer() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let buffer = CanvasDepthStencilBuffer::new(
            &instance,
            &CanvasDepthStencilBufferDescriptor {
                sample_count: 2,
                format: CanvasDepthStencilBufferFormat::Depth32Float,
                size: CanvasSize::new(12, 20),
            },
        );

        expect_that!(&buffer.sample_count(), eq(2));
        expect_that!(
            &buffer.format(),
            eq(CanvasDepthStencilBufferFormat::Depth32Float)
        );
        expect_that!(buffer.size(), eq(CanvasSize::new(12, 20)));

        let reference = buffer.reference();
        expect_that!(&reference.sample_count(), eq(2));
        expect_that!(
            &reference.format(),
            eq(CanvasDepthStencilBufferFormat::Depth32Float)
        );
    }

    #[test]
    #[serial_test::serial]
    fn canvas_buffer() {
        let event_loop = EventLoop::<()>::new_any_thread();
        let window = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        let (instance, surface) = unsafe {
            Instance::new_with_compatible_window(&InstanceDescriptor::default(), &window).unwrap()
        };

        let mut buffer = CanvasBuffer::new(
            &instance,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                swap_chain_descriptor: Some(CanvasBufferSwapChainDescriptor {
                    surface: surface,
                    format: CanvasColorBufferFormat::default(),
                }),
                color_buffer_descriptors: vec![
                    CanvasBufferColorBufferDescriptor {
                        format: CanvasColorBufferFormat::default(),
                        usage: CanvasColorBufferUsage::empty(),
                    },
                    CanvasBufferColorBufferDescriptor {
                        format: CanvasColorBufferFormat::Bgra8Unorm,
                        usage: CanvasColorBufferUsage::empty(),
                    },
                ],
                depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
            },
        );

        expect_that!(buffer.size(), eq(CanvasSize::new(12, 20)));
        expect_that!(&buffer.sample_count(), eq(2));
        expect_that!(buffer.swap_chain().is_some());
        expect_that!(&buffer.color_buffers.len(), eq(2));
        expect_that!(buffer.depth_stencil_buffer().is_some());

        {
            let swap_chain = buffer.swap_chain().unwrap();
            expect_that!(&swap_chain.sample_count(), eq(2));
            expect_that!(&swap_chain.format(), eq(CanvasColorBufferFormat::default()));
            expect_that!(swap_chain.size(), eq(CanvasSize::new(12, 20)));
        }

        {
            let buffer = &buffer.color_buffers()[0];
            expect_that!(&buffer.sample_count(), eq(2));
            expect_that!(&buffer.format(), eq(CanvasColorBufferFormat::default()));
            expect_that!(buffer.size(), eq(CanvasSize::new(12, 20)));
        }

        {
            let buffer = &buffer.color_buffers()[1];
            expect_that!(&buffer.sample_count(), eq(2));
            expect_that!(&buffer.format(), eq(CanvasColorBufferFormat::Bgra8Unorm));
            expect_that!(buffer.size(), eq(CanvasSize::new(12, 20)));
        }

        {
            let buffer = buffer.depth_stencil_buffer().unwrap();
            expect_that!(&buffer.sample_count(), eq(2));
            expect_that!(
                &buffer.format(),
                eq(CanvasDepthStencilBufferFormat::Depth32Float)
            );
            expect_that!(buffer.size(), eq(CanvasSize::new(12, 20)));
        }

        {
            let frame = buffer.current_frame().unwrap();
            expect_that!(frame.swap_chain().is_some());
            expect_that!(&frame.color_buffers().len(), eq(2));
            expect_that!(frame.depth_stencil_buffer.is_some());
        }
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "No buffer defined for a canvas buffer")]
    fn canvas_buffer_creation_error() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let _buffer = CanvasBuffer::new(
            &instance,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                swap_chain_descriptor: None,
                color_buffer_descriptors: Vec::new(),
                depth_stencil_buffer_format: None,
            },
        );
    }
}
