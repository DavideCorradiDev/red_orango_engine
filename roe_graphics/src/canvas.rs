use super::{
    Extent3d, Instance, PresentMode, SampleCount, Size, Surface, SurfaceConfiguration,
    SurfaceError, SurfaceTexture, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsage, TextureView, TextureViewDescriptor, TextureViewDimension,
};

// TODO: add tests for "invalid" canvas buffer.

fn canvas_texture_descriptor<'a>(
    size: CanvasSize,
    sample_count: SampleCount,
    format: TextureFormat,
    usage: TextureUsage,
) -> TextureDescriptor<'a> {
    TextureDescriptor {
        label: None,
        size: Extent3d {
            width: size.width(),
            height: size.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        format,
        dimension: TextureDimension::D2,
        usage: usage | TextureUsage::RENDER_ATTACHMENT,
    }
}

fn canvas_texture_view_descriptor<'a>(format: TextureFormat) -> TextureViewDescriptor<'a> {
    TextureViewDescriptor {
        label: None,
        format: Some(format),
        dimension: Some(TextureViewDimension::D2),
        aspect: TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    }
}

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
pub struct CanvasSurfaceRef<'a> {
    sample_count: SampleCount,
    format: CanvasColorBufferFormat,
    multisampled_buffer: Option<&'a TextureView>,
    surface_texture: SurfaceTexture,
    surface_view: TextureView,
}

impl<'a> CanvasSurfaceRef<'a> {
    pub fn attachment(&self) -> &TextureView {
        match self.multisampled_buffer {
            Some(v) => &v,
            None => &self.surface_view,
        }
    }

    pub fn resolve_target(&self) -> Option<&TextureView> {
        match self.multisampled_buffer {
            Some(_) => Some(&self.surface_view),
            None => None,
        }
    }

    pub fn sample_count(&self) -> SampleCount {
        self.sample_count
    }

    pub fn format(&self) -> CanvasColorBufferFormat {
        self.format
    }

    pub fn present(self) {
        self.surface_texture.present()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CanvasSurfaceDescriptor {
    pub size: CanvasSize,
    pub sample_count: SampleCount,
    pub format: CanvasColorBufferFormat,
}

#[derive(Debug)]
pub struct CanvasSurface {
    size: CanvasSize,
    sample_count: SampleCount,
    format: CanvasColorBufferFormat,
    multisampled_buffer: Option<TextureView>,
    surface: Surface,
}

impl CanvasSurface {
    pub fn new(surface: Surface) -> Self {
        Self {
            size: CanvasSize::new(0, 0),
            sample_count: 1,
            format: CanvasColorBufferFormat::default(),
            multisampled_buffer: None,
            surface,
        }
    }

    pub fn configure(&mut self, instance: &Instance, desc: &CanvasSurfaceDescriptor) {
        let format = TextureFormat::from(desc.format);
        self.surface.configure(
            instance,
            &SurfaceConfiguration {
                usage: TextureUsage::RENDER_ATTACHMENT,
                format,
                width: desc.size.width(),
                height: desc.size.height(),
                present_mode: PresentMode::Mailbox,
            },
        );
        self.multisampled_buffer = if desc.sample_count > 1 {
            let multisampling_buffer_texture = Texture::new(
                instance,
                &canvas_texture_descriptor(
                    desc.size,
                    desc.sample_count,
                    format,
                    TextureUsage::empty(),
                ),
            );
            Some(multisampling_buffer_texture.create_view(&canvas_texture_view_descriptor(format)))
        } else {
            None
        };
        self.size = desc.size;
        self.sample_count = desc.sample_count;
        self.format = desc.format;
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

    pub fn reference(&mut self) -> Result<CanvasSurfaceRef, SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        let surface_view = surface_texture
            .texture
            .create_view(&canvas_texture_view_descriptor(TextureFormat::from(
                self.format,
            )));
        let multisampled_buffer = match self.multisampled_buffer {
            Some(ref v) => Some(v),
            None => None,
        };
        Ok(CanvasSurfaceRef {
            sample_count: self.sample_count,
            format: self.format,
            multisampled_buffer,
            surface_texture,
            surface_view,
        })
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
        let format = TextureFormat::from(desc.format);
        let mut tex_desc =
            canvas_texture_descriptor(desc.size, 1, format, TextureUsage::from(desc.usage));
        let tex_view_desc = canvas_texture_view_descriptor(format);

        let main_buffer_texture = Texture::new(instance, &tex_desc);
        let main_buffer_view = main_buffer_texture.create_view(&tex_view_desc);

        let multisampled_buffer = if desc.sample_count > 1 {
            tex_desc.sample_count = desc.sample_count;
            Some(Texture::new(instance, &tex_desc).create_view(&tex_view_desc))
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
        let format = TextureFormat::from(desc.format);
        let buffer_texture = Texture::new(
            instance,
            &canvas_texture_descriptor(desc.size, desc.sample_count, format, TextureUsage::empty()),
        );
        let buffer_view = buffer_texture.create_view(&canvas_texture_view_descriptor(format));
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
    surface: Option<CanvasSurfaceRef<'a>>,
    color_buffers: Vec<CanvasColorBufferRef<'a>>,
    depth_stencil_buffer: Option<CanvasDepthStencilBufferRef<'a>>,
}

impl<'a> CanvasFrame<'a> {
    pub fn surface(&self) -> Option<&CanvasSurfaceRef<'a>> {
        self.surface.as_ref()
    }

    pub fn color_buffers(&self) -> &Vec<CanvasColorBufferRef<'a>> {
        &self.color_buffers
    }

    pub fn depth_stencil_buffer(&self) -> Option<&CanvasDepthStencilBufferRef<'a>> {
        self.depth_stencil_buffer.as_ref()
    }

    pub fn present(self) {
        if let Some(sc) = self.surface {
            sc.present();
        }
    }
}

#[derive(Debug)]
pub struct CanvasBufferSurfaceDescriptor {
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
    pub surface_descriptor: Option<CanvasBufferSurfaceDescriptor>,
    pub color_buffer_descriptors: Vec<CanvasBufferColorBufferDescriptor>,
    pub depth_stencil_buffer_format: Option<CanvasDepthStencilBufferFormat>,
}

#[derive(Debug)]
pub struct CanvasBuffer {
    size: CanvasSize,
    sample_count: SampleCount,
    // rename similar varialbes
    canvas_surface: Option<CanvasSurface>,
    canvas_color_buffers: Vec<CanvasColorBuffer>,
    canvas_depth_stencil_buffer: Option<CanvasDepthStencilBuffer>,
}

impl CanvasBuffer {
    pub fn new(
        // TODO: make sure passing surface as optional is tested properly.
        instance: &Instance,
        surface: Option<Surface>,
        desc: &CanvasBufferDescriptor,
    ) -> Self {
        // TODO: should we make sure that if surface is not passed, also descriptor is not passed?
        let canvas_surface = match surface {
            Some(surface) => Some(CanvasSurface::new(surface)),
            None => None,
        };
        let mut canvas_buffer = Self {
            size: desc.size,
            sample_count: desc.sample_count,
            canvas_surface,
            canvas_color_buffers: Vec::new(),
            canvas_depth_stencil_buffer: None,
        };
        canvas_buffer.configure(instance, desc);
        canvas_buffer
    }

    // TODO: test.
    pub fn configure(&mut self, instance: &Instance, desc: &CanvasBufferDescriptor) {
        self.size = desc.size;
        self.sample_count = desc.sample_count;

        // If the requested surface area is 0, don't actually reconfigure (it won't work).
        // Instead, only the variables above are set, so that the canvas buffer can currently be
        // flagged as "invalid", no frames are returned, and the surface will be recreated when the
        // size changes again to a valid value.
        if desc.size.width() == 0 || desc.size.height() == 0 {
            return;
        }

        self.configure_canvas_surface(instance, desc);
        self.canvas_color_buffers = Self::create_color_buffers(instance, desc);
        self.canvas_depth_stencil_buffer = Self::create_depth_stencil_buffer(instance, desc);
        self.assert_has_buffer();
    }

    fn configure_canvas_surface(&mut self, instance: &Instance, desc: &CanvasBufferDescriptor) {
        match &mut self.canvas_surface {
            Some(canvas_surface) => {
                let format = match &desc.surface_descriptor {
                    Some(sd) => sd.format,
                    None => {
                        panic!("Canvas buffer created with a surface, but no surface descriptor")
                    }
                };
                canvas_surface.configure(
                    instance,
                    &CanvasSurfaceDescriptor {
                        size: desc.size,
                        sample_count: desc.sample_count,
                        format,
                    },
                );
            },
            None => {
                if let Some(_) = &desc.surface_descriptor {
                    panic!("Canvas buffer created with a surface descriptor, but no surface")
                }
            }
        }
    }

    fn create_color_buffers(
        instance: &Instance,
        desc: &CanvasBufferDescriptor,
    ) -> Vec<CanvasColorBuffer> {
        let mut canvas_color_buffers = Vec::with_capacity(desc.color_buffer_descriptors.len());
        for cbd in desc.color_buffer_descriptors.iter() {
            canvas_color_buffers.push(CanvasColorBuffer::new(
                instance,
                &CanvasColorBufferDescriptor {
                    size: desc.size,
                    sample_count: desc.sample_count,
                    format: cbd.format,
                    usage: cbd.usage,
                },
            ));
        }
        canvas_color_buffers
    }

    fn create_depth_stencil_buffer(
        instance: &Instance,
        desc: &CanvasBufferDescriptor,
    ) -> Option<CanvasDepthStencilBuffer> {
        match &desc.depth_stencil_buffer_format {
            Some(format) => Some(CanvasDepthStencilBuffer::new(
                instance,
                &CanvasDepthStencilBufferDescriptor {
                    size: desc.size,
                    sample_count: desc.sample_count,
                    format: *format,
                },
            )),
            None => None,
        }
    }

    fn assert_has_buffer(&self) {
        assert!(
            self.canvas_surface.is_some()
                || !self.canvas_color_buffers.is_empty()
                || self.canvas_depth_stencil_buffer.is_some(),
            "No buffer defined for a canvas buffer"
        );
    }

    pub fn is_valid(&self) -> bool {
        self.size.width() != 0 && self.size.height() != 0
    }

    pub fn size(&self) -> &CanvasSize {
        &self.size
    }

    pub fn sample_count(&self) -> SampleCount {
        self.sample_count
    }

    pub fn surface(&self) -> Option<&CanvasSurface> {
        self.canvas_surface.as_ref()
    }

    pub fn color_buffers(&self) -> &[CanvasColorBuffer] {
        self.canvas_color_buffers.as_slice()
    }

    pub fn depth_stencil_buffer(&self) -> Option<&CanvasDepthStencilBuffer> {
        self.canvas_depth_stencil_buffer.as_ref()
    }

    pub fn current_frame(&mut self) -> Result<Option<CanvasFrame>, SurfaceError> {
        if !self.is_valid() {
            return Ok(None);
        }

        let surface = match &mut self.canvas_surface {
            Some(surface) => Some(surface.reference()?),
            None => None,
        };

        let mut color_buffers = Vec::with_capacity(self.canvas_color_buffers.len());
        for color_buffer in self.canvas_color_buffers.iter() {
            color_buffers.push(color_buffer.reference());
        }

        let depth_stencil_buffer = match &self.canvas_depth_stencil_buffer {
            Some(depth_stencil_buffer) => Some(depth_stencil_buffer.reference()),
            None => None,
        };

        Ok(Some(CanvasFrame {
            surface,
            color_buffers,
            depth_stencil_buffer,
        }))
    }
}

pub trait Canvas {
    fn current_frame(&mut self) -> Result<Option<CanvasFrame>, SurfaceError>;
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
    fn canvas_surface() {
        let event_loop = EventLoop::<()>::new_any_thread();
        let window = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        let (_instance, surface) = unsafe {
            Instance::new_with_compatible_window(&InstanceDescriptor::default(), &window).unwrap()
        };

        let surface = CanvasSurface::new(surface);

        expect_that!(&surface.sample_count(), eq(1));
        expect_that!(&surface.format(), eq(CanvasColorBufferFormat::default()));
        expect_that!(surface.size(), eq(CanvasSize::new(0, 0)));
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "Surface was not configured")]
    fn retrieve_canvas_surface_reference_before_configuration() {
        let event_loop = EventLoop::<()>::new_any_thread();
        let window = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        let (_instance, surface) = unsafe {
            Instance::new_with_compatible_window(&InstanceDescriptor::default(), &window).unwrap()
        };

        let mut surface = CanvasSurface::new(surface);
        surface.reference().unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn canvas_surface_configure() {
        let event_loop = EventLoop::<()>::new_any_thread();
        let window = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        let (instance, surface) = unsafe {
            Instance::new_with_compatible_window(&InstanceDescriptor::default(), &window).unwrap()
        };

        let mut surface = CanvasSurface::new(surface);
        surface.configure(
            &instance,
            &CanvasSurfaceDescriptor {
                sample_count: 2,
                format: CanvasColorBufferFormat::Bgra8Unorm,
                size: CanvasSize::new(12, 20),
            },
        );

        expect_that!(&surface.sample_count(), eq(2));
        expect_that!(&surface.format(), eq(CanvasColorBufferFormat::Bgra8Unorm));
        expect_that!(surface.size(), eq(CanvasSize::new(12, 20)));

        let reference = surface.reference().unwrap();
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
            Some(surface),
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                surface_descriptor: Some(CanvasBufferSurfaceDescriptor {
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
        expect_that!(buffer.surface().is_some());
        expect_that!(&buffer.color_buffers().len(), eq(2));
        expect_that!(buffer.depth_stencil_buffer().is_some());

        {
            let surface = buffer.surface().unwrap();
            expect_that!(&surface.sample_count(), eq(2));
            expect_that!(&surface.format(), eq(CanvasColorBufferFormat::default()));
            expect_that!(surface.size(), eq(CanvasSize::new(12, 20)));
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
            let frame = buffer.current_frame().unwrap().unwrap();
            expect_that!(frame.surface().is_some());
            expect_that!(&frame.color_buffers().len(), eq(2));
            expect_that!(frame.depth_stencil_buffer.is_some());
        }
    }

    #[test]
    #[serial_test::serial]
    fn canvas_buffer_only_surface() {
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
            Some(surface),
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                surface_descriptor: Some(CanvasBufferSurfaceDescriptor {
                    format: CanvasColorBufferFormat::default(),
                }),
                color_buffer_descriptors: vec![],
                depth_stencil_buffer_format: None,
            },
        );

        expect_that!(buffer.size(), eq(CanvasSize::new(12, 20)));
        expect_that!(&buffer.sample_count(), eq(2));
        expect_that!(buffer.surface().is_some());
        expect_that!(&buffer.color_buffers().len(), eq(0));
        expect_that!(buffer.depth_stencil_buffer().is_none());

        {
            let surface = buffer.surface().unwrap();
            expect_that!(&surface.sample_count(), eq(2));
            expect_that!(&surface.format(), eq(CanvasColorBufferFormat::default()));
            expect_that!(surface.size(), eq(CanvasSize::new(12, 20)));
        }

        {
            let frame = buffer.current_frame().unwrap().unwrap();
            expect_that!(frame.surface().is_some());
            expect_that!(&frame.color_buffers().len(), eq(0));
            expect_that!(frame.depth_stencil_buffer.is_none());
        }
    }

    #[test]
    #[serial_test::serial]
    fn canvas_buffer_only_color() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let mut buffer = CanvasBuffer::new(
            &instance,
            None,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                surface_descriptor: None,
                color_buffer_descriptors: vec![CanvasBufferColorBufferDescriptor {
                    format: CanvasColorBufferFormat::default(),
                    usage: CanvasColorBufferUsage::empty(),
                }],
                depth_stencil_buffer_format: None,
            },
        );

        expect_that!(buffer.size(), eq(CanvasSize::new(12, 20)));
        expect_that!(&buffer.sample_count(), eq(2));
        expect_that!(buffer.surface().is_none());
        expect_that!(&buffer.color_buffers().len(), eq(1));
        expect_that!(buffer.depth_stencil_buffer().is_none());

        {
            let buffer = &buffer.color_buffers()[0];
            expect_that!(&buffer.sample_count(), eq(2));
            expect_that!(&buffer.format(), eq(CanvasColorBufferFormat::default()));
            expect_that!(buffer.size(), eq(CanvasSize::new(12, 20)));
        }

        {
            let frame = buffer.current_frame().unwrap().unwrap();
            expect_that!(frame.surface().is_none());
            expect_that!(&frame.color_buffers().len(), eq(1));
            expect_that!(frame.depth_stencil_buffer.is_none());
        }
    }

    #[test]
    #[serial_test::serial]
    fn canvas_buffer_only_depth_stencil() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let mut buffer = CanvasBuffer::new(
            &instance,
            None,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                surface_descriptor: None,
                color_buffer_descriptors: vec![],
                depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
            },
        );

        expect_that!(buffer.size(), eq(CanvasSize::new(12, 20)));
        expect_that!(&buffer.sample_count(), eq(2));
        expect_that!(buffer.surface().is_none());
        expect_that!(&buffer.color_buffers().len(), eq(0));
        expect_that!(buffer.depth_stencil_buffer().is_some());

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
            let frame = buffer.current_frame().unwrap().unwrap();
            expect_that!(frame.surface().is_none());
            expect_that!(&frame.color_buffers().len(), eq(0));
            expect_that!(frame.depth_stencil_buffer.is_some());
        }
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "No buffer defined for a canvas buffer")]
    fn canvas_buffer_error_no_buffers() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let _buffer = CanvasBuffer::new(
            &instance,
            None,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                surface_descriptor: None,
                color_buffer_descriptors: Vec::new(),
                depth_stencil_buffer_format: None,
            },
        );
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "Canvas buffer created with a surface, but no surface descriptor")]
    fn canvas_buffer_error_no_surface_descriptor() {
        let event_loop = EventLoop::<()>::new_any_thread();
        let window = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        let (instance, surface) = unsafe {
            Instance::new_with_compatible_window(&InstanceDescriptor::default(), &window).unwrap()
        };
        CanvasBuffer::new(
            &instance,
            Some(surface),
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                surface_descriptor: None,
                color_buffer_descriptors: vec![],
                depth_stencil_buffer_format: None,
            },
        );
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "Canvas buffer created with a surface descriptor, but no surface")]
    fn canvas_buffer_error_no_surface() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        CanvasBuffer::new(
            &instance,
            None,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(12, 20),
                sample_count: 2,
                surface_descriptor: Some(CanvasBufferSurfaceDescriptor {
                    format: CanvasColorBufferFormat::default(),
                }),
                color_buffer_descriptors: vec![],
                depth_stencil_buffer_format: None,
            },
        );
    }
}
