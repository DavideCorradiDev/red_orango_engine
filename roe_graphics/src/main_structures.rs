use super::{
    AdapterInfo, Backend, BindGroupDescriptor, BindGroupLayoutDescriptor, BufferAddress,
    BufferDescriptor, BufferInitDescriptor, BufferUsage, ColorF64, CommandBuffer,
    CommandEncoderDescriptor, Extent3d, Features, ImageCopyBuffer, ImageCopyTexture,
    ImageDataLayout, Limits, Maintain, MapMode, Operations, Origin3d, PipelineLayoutDescriptor,
    PowerPreference, RenderBundleEncoderDescriptor, RenderPipelineDescriptor, SamplerDescriptor,
    ShaderModuleDescriptor, SurfaceConfiguration, SurfaceError, SurfaceTexture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsage,
};

use roe_os as os;

use wgpu::util::DeviceExt;

use raw_window_handle::HasRawWindowHandle;

use std::{
    default::Default,
    ops::{Deref, DerefMut},
};

pub type SampleCount = u32;
pub type ColorOperations = Operations<ColorF64>;
pub type DepthOperations = Operations<f32>;
pub type StencilOperations = Operations<u32>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InstanceDescriptor {
    pub backend: Backend,
    pub power_preference: PowerPreference,
    pub required_features: Features,
    pub optional_features: Features,
    pub required_limits: Limits,
}

impl InstanceDescriptor {
    pub fn high_performance() -> Self {
        let mut required_limits = Limits::default();
        required_limits.max_push_constant_size = 128;
        Self {
            backend: Backend::PRIMARY,
            power_preference: PowerPreference::HighPerformance,
            required_features: Features::default() | Features::PUSH_CONSTANTS,
            optional_features: Features::empty(),
            required_limits,
        }
    }
}

impl Default for InstanceDescriptor {
    fn default() -> Self {
        let mut required_limits = Limits::default();
        required_limits.max_push_constant_size = 128;
        Self {
            backend: Backend::PRIMARY,
            power_preference: PowerPreference::HighPerformance,
            required_features: Features::default() | Features::PUSH_CONSTANTS,
            optional_features: Features::empty(),
            required_limits,
        }
    }
}

#[derive(Debug)]
pub struct Instance {
    queue: wgpu::Queue,
    device: wgpu::Device,
    adapter: wgpu::Adapter,
    instance: wgpu::Instance,
}

impl Instance {
    pub fn new(desc: &InstanceDescriptor) -> Result<Self, InstanceCreationError> {
        let instance = Self::create_instance(desc);
        let adapter = Self::create_adapter(&instance, desc, None)?;
        let (device, queue) = Self::create_device_and_queue(&adapter, desc)?;
        Ok(Self {
            queue,
            adapter,
            device,
            instance,
        })
    }

    // Unsafe: surface creation.
    pub unsafe fn new_with_compatible_window(
        desc: &InstanceDescriptor,
        compatible_window: &os::Window,
    ) -> Result<(Self, Surface), InstanceCreationError> {
        let instance = Self::create_instance(desc);
        let surface = instance.create_surface(compatible_window);
        let adapter = Self::create_adapter(&instance, desc, Some(&surface))?;
        let (device, queue) = Self::create_device_and_queue(&adapter, desc)?;
        Ok((
            Self {
                queue,
                adapter,
                device,
                instance,
            },
            Surface { value: surface },
        ))
    }

    pub fn info(&self) -> AdapterInfo {
        self.adapter.get_info()
    }

    pub fn poll(&self, maintain: Maintain) {
        self.device.poll(maintain)
    }

    pub fn features(&self) -> Features {
        self.device.features()
    }

    pub fn limits(&self) -> Limits {
        self.device.limits()
    }

    pub fn submit<I: IntoIterator<Item = CommandBuffer>>(&self, command_buffers: I) {
        self.queue.submit(command_buffers);
    }

    pub fn write_buffer(&self, buffer: &Buffer, offset: BufferAddress, data: &[u8]) {
        self.queue.write_buffer(buffer, offset, data);
    }

    pub fn write_texture(
        &self,
        texture: ImageCopyTexture<'_>,
        data: &[u8],
        data_layout: ImageDataLayout,
        size: Extent3d,
    ) {
        self.queue.write_texture(texture, data, data_layout, size);
    }

    fn create_instance(desc: &InstanceDescriptor) -> wgpu::Instance {
        wgpu::Instance::new(desc.backend)
    }

    fn create_adapter(
        instance: &wgpu::Instance,
        desc: &InstanceDescriptor,
        compatible_surface: Option<&wgpu::Surface>,
    ) -> Result<wgpu::Adapter, InstanceCreationError> {
        let adapter = match futures::executor::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: desc.power_preference,
                compatible_surface,
                force_fallback_adapter: false,
            },
        )) {
            Some(v) => v,
            None => return Err(InstanceCreationError::AdapterRequestFailed),
        };

        if !adapter.features().contains(desc.required_features) {
            return Err(InstanceCreationError::FeaturesNotAvailable(
                desc.required_features - adapter.features(),
            ));
        }

        Ok(adapter)
    }

    fn create_device_and_queue(
        adapter: &wgpu::Adapter,
        desc: &InstanceDescriptor,
    ) -> Result<(wgpu::Device, wgpu::Queue), InstanceCreationError> {
        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: (desc.optional_features & adapter.features()) | desc.required_features,
                limits: desc.required_limits.clone(),
                label: None,
            },
            None,
        ))?;
        Ok((device, queue))
    }
}

#[derive(Debug)]
pub struct Surface {
    value: wgpu::Surface,
}

impl Surface {
    pub unsafe fn new<W: HasRawWindowHandle>(instance: &Instance, window: &W) -> Self {
        Self {
            value: instance.instance.create_surface(window),
        }
    }

    pub fn configure(&self, instance: &Instance, config: &SurfaceConfiguration) {
        self.value.configure(&instance.device, config);
    }

    pub fn get_current_texture(&self) -> Result<SurfaceTexture, SurfaceError> {
        self.value.get_current_texture()
    }
}

impl Deref for Surface {
    type Target = wgpu::Surface;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug)]
pub struct ShaderModule {
    value: wgpu::ShaderModule,
}

impl ShaderModule {
    pub fn new(instance: &Instance, desc: &ShaderModuleDescriptor) -> Self {
        Self {
            value: instance.device.create_shader_module(desc),
        }
    }
}

impl Deref for ShaderModule {
    type Target = wgpu::ShaderModule;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for ShaderModule {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug)]
pub struct PipelineLayout {
    value: wgpu::PipelineLayout,
}

impl PipelineLayout {
    pub fn new(instance: &Instance, desc: &PipelineLayoutDescriptor) -> Self {
        Self {
            value: instance.device.create_pipeline_layout(desc),
        }
    }
}

impl Deref for PipelineLayout {
    type Target = wgpu::PipelineLayout;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for PipelineLayout {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug)]
pub struct RenderPipeline {
    value: wgpu::RenderPipeline,
}

impl RenderPipeline {
    pub fn new(instance: &Instance, desc: &RenderPipelineDescriptor) -> Self {
        Self {
            value: instance.device.create_render_pipeline(desc),
        }
    }
}

impl Deref for RenderPipeline {
    type Target = wgpu::RenderPipeline;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for RenderPipeline {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug)]
pub struct Buffer {
    value: wgpu::Buffer,
}

impl Buffer {
    pub fn new(instance: &Instance, desc: &BufferDescriptor) -> Self {
        Self {
            value: instance.device.create_buffer(desc),
        }
    }

    pub fn init(instance: &Instance, desc: &BufferInitDescriptor) -> Self {
        Self {
            value: instance.device.create_buffer_init(desc),
        }
    }
}

impl Deref for Buffer {
    type Target = wgpu::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug)]
pub struct CommandEncoder {
    value: wgpu::CommandEncoder,
}

impl CommandEncoder {
    pub fn new(instance: &Instance, desc: &CommandEncoderDescriptor) -> Self {
        Self {
            value: instance.device.create_command_encoder(desc),
        }
    }

    pub fn finish(self) -> CommandBuffer {
        self.value.finish()
    }
}

impl Deref for CommandEncoder {
    type Target = wgpu::CommandEncoder;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for CommandEncoder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug)]
pub struct RenderBundleEncoder<'a> {
    value: wgpu::RenderBundleEncoder<'a>,
}

impl<'a> RenderBundleEncoder<'a> {
    pub fn new(instance: &'a Instance, desc: &RenderBundleEncoderDescriptor) -> Self {
        Self {
            value: instance.device.create_render_bundle_encoder(desc),
        }
    }
}

impl<'a> Deref for RenderBundleEncoder<'a> {
    type Target = wgpu::RenderBundleEncoder<'a>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a> DerefMut for RenderBundleEncoder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug)]
pub struct BindGroupLayout {
    value: wgpu::BindGroupLayout,
}

impl BindGroupLayout {
    pub fn new(instance: &Instance, desc: &BindGroupLayoutDescriptor) -> Self {
        Self {
            value: instance.device.create_bind_group_layout(desc),
        }
    }
}

impl Deref for BindGroupLayout {
    type Target = wgpu::BindGroupLayout;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for BindGroupLayout {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug)]
pub struct BindGroup {
    value: wgpu::BindGroup,
}

impl BindGroup {
    pub fn new(instance: &Instance, desc: &BindGroupDescriptor) -> Self {
        Self {
            value: instance.device.create_bind_group(desc),
        }
    }
}

impl Deref for BindGroup {
    type Target = wgpu::BindGroup;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for BindGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct TextureBufferSize {
    width: u64,
    height: u64,
    unpadded_bytes_per_row: u64,
    padded_bytes_per_row: u64,
}

impl TextureBufferSize {
    pub fn new(width: u64, height: u64) -> Self {
        let bytes_per_pixel = std::mem::size_of::<u32>() as u64;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }

    pub fn byte_count(&self) -> u64 {
        self.padded_bytes_per_row * self.height
    }
}

#[derive(Debug)]
pub struct Texture {
    value: wgpu::Texture,
    size: Extent3d,
}

impl Texture {
    pub fn new(instance: &Instance, desc: &TextureDescriptor) -> Self {
        Self {
            value: instance.device.create_texture(desc),
            size: desc.size,
        }
    }

    pub fn size(&self) -> &Extent3d {
        &self.size
    }

    pub fn from_image(instance: &Instance, img: &image::RgbaImage, usage: TextureUsage) -> Self {
        let img_dimensions = img.dimensions();
        let size = Extent3d {
            width: img_dimensions.0,
            height: img_dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = Self::new(
            instance,
            &TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: usage | TextureUsage::COPY_DST,
            },
        );
        texture.write(
            instance,
            0,
            Origin3d::ZERO,
            img.as_flat_samples().as_slice(),
            ImageDataLayout {
                offset: 0,
                bytes_per_row: core::num::NonZeroU32::new(4 * size.width),
                rows_per_image: None,
            },
            size,
        );
        texture
    }

    pub fn to_image(&self, instance: &Instance) -> image::RgbaImage {
        let buffer_size = TextureBufferSize::new(self.size.width as u64, self.size.height as u64);
        let output_buffer = Buffer::new(
            instance,
            &BufferDescriptor {
                label: None,
                size: buffer_size.byte_count(),
                usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let mut encoder = CommandEncoder::new(instance, &CommandEncoderDescriptor::default());
        {
            encoder.copy_texture_to_buffer(
                ImageCopyTexture {
                    texture: &self.value,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                ImageCopyBuffer {
                    buffer: &output_buffer,
                    layout: ImageDataLayout {
                        offset: 0,
                        bytes_per_row: core::num::NonZeroU32::new(
                            buffer_size.padded_bytes_per_row as u32,
                        ),
                        rows_per_image: None,
                    },
                },
                *self.size(),
            );
        }
        instance.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        let buffer_future = buffer_slice.map_async(MapMode::Read);
        instance.poll(Maintain::Wait);

        let future_image = async {
            buffer_future.await.unwrap();
            let padded_buffer = buffer_slice.get_mapped_range();

            let mut unpadded_buffer =
                Vec::with_capacity((self.size.width * self.size.height) as usize);
            for chunk in padded_buffer.chunks(buffer_size.padded_bytes_per_row as usize) {
                unpadded_buffer
                    .extend_from_slice(&chunk[..buffer_size.unpadded_bytes_per_row as usize]);
            }
            let image =
                image::RgbaImage::from_raw(self.size.width, self.size.height, unpadded_buffer)
                    .unwrap();

            drop(padded_buffer);
            output_buffer.unmap();
            image
        };

        futures::executor::block_on(future_image)
    }

    pub fn write(
        &self,
        instance: &Instance,
        mip_level: u32,
        origin: Origin3d,
        data: &[u8],
        data_layout: ImageDataLayout,
        size: Extent3d,
    ) {
        instance.write_texture(
            ImageCopyTexture {
                texture: self,
                mip_level,
                origin,
                aspect: TextureAspect::All,
            },
            data,
            data_layout,
            size,
        );
    }
}

impl Deref for Texture {
    type Target = wgpu::Texture;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Texture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug)]
pub struct Sampler {
    value: wgpu::Sampler,
}

impl Sampler {
    pub fn new(instance: &Instance, desc: &SamplerDescriptor) -> Self {
        Self {
            value: instance.device.create_sampler(desc),
        }
    }
}

impl Deref for Sampler {
    type Target = wgpu::Sampler;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Sampler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstanceCreationError {
    AdapterRequestFailed,
    FeaturesNotAvailable(Features),
    DeviceRequestFailed(wgpu::RequestDeviceError),
}

impl std::fmt::Display for InstanceCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceCreationError::AdapterRequestFailed => write!(f, "Adapter request failed"),
            InstanceCreationError::FeaturesNotAvailable(features) => {
                write!(f, "Required features are not available ({:?})", features)
            }
            InstanceCreationError::DeviceRequestFailed(e) => {
                write!(f, "Device request failed ({})", e)
            }
        }
    }
}

impl std::error::Error for InstanceCreationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InstanceCreationError::DeviceRequestFailed(e) => Some(e),
            _ => None,
        }
    }
}

impl From<wgpu::RequestDeviceError> for InstanceCreationError {
    fn from(e: wgpu::RequestDeviceError) -> Self {
        InstanceCreationError::DeviceRequestFailed(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};
    use os::EventLoopAnyThread;

    #[test]
    #[serial_test::serial]
    fn default_config() {
        let _instance = Instance::new(&InstanceDescriptor::default()).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn new() {
        let _instance = Instance::new(&InstanceDescriptor {
            backend: Backend::VULKAN,
            power_preference: PowerPreference::HighPerformance,
            required_features: Features::default(),
            optional_features: Features::empty(),
            required_limits: Limits::default(),
        })
        .unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn new_with_compatible_window() {
        let event_loop = os::EventLoop::<()>::new_any_thread();
        let window = os::WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        let (_instance, _surface) = unsafe {
            Instance::new_with_compatible_window(&InstanceDescriptor::default(), &window).unwrap()
        };
    }

    #[test]
    #[serial_test::serial]
    fn load_texture_from_image() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let image = image::open("data/pictures/test.png").unwrap().into_rgba8();
        let texture = Texture::from_image(&instance, &image, TextureUsage::TEXTURE_BINDING);
        expect_that!(
            &texture.size(),
            eq(Extent3d {
                width: 27,
                height: 33,
                depth_or_array_layers: 1
            })
        );
    }
}
