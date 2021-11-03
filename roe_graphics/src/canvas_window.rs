use std::default::Default;

use roe_app::{
    event::EventLoop,
    window,
    window::{ExternalError, NotSupportedError, OsError, Window, WindowId},
};

use super::{
    Canvas, CanvasBuffer, CanvasBufferDescriptor, CanvasBufferSwapChainDescriptor,
    CanvasColorBufferFormat, CanvasDepthStencilBufferFormat, CanvasFrame, CanvasSize, Instance,
    SampleCount, Surface, SwapChainError,
};

#[derive(Debug, PartialEq, Clone)]
pub struct CanvasWindowDescriptor {
    pub sample_count: SampleCount,
    pub color_buffer_format: CanvasColorBufferFormat,
    pub depth_stencil_buffer_format: Option<CanvasDepthStencilBufferFormat>,
}

impl Default for CanvasWindowDescriptor {
    fn default() -> Self {
        Self {
            sample_count: 1,
            color_buffer_format: CanvasColorBufferFormat::default(),
            depth_stencil_buffer_format: None,
        }
    }
}

#[derive(Debug)]
pub struct CanvasWindow {
    canvas_buffer: CanvasBuffer,
    surface: Surface,
    window: Window,
}

impl CanvasWindow {
    // Unsafe: surface creation.
    pub unsafe fn new<T: 'static>(
        instance: &Instance,
        event_loop: &EventLoop<T>,
        desc: &CanvasWindowDescriptor,
    ) -> Result<Self, OsError> {
        let window = Window::new(event_loop)?;
        Ok(Self::from_window(instance, window, desc))
    }

    // Unsafe: surface creation.
    pub unsafe fn from_window(
        instance: &Instance,
        window: Window,
        desc: &CanvasWindowDescriptor,
    ) -> Self {
        let surface = Surface::new(&instance, &window);
        Self::from_window_and_surface(instance, window, surface, desc)
    }

    // Unsafe: surface must correspond to the window.
    pub unsafe fn from_window_and_surface(
        instance: &Instance,
        window: Window,
        surface: Surface,
        desc: &CanvasWindowDescriptor,
    ) -> Self {
        let surface_size = window.inner_size();
        let canvas_buffer = CanvasBuffer::new(
            instance,
            &CanvasBufferDescriptor {
                size: CanvasSize::new(surface_size.width, surface_size.height),
                sample_count: desc.sample_count,
                swap_chain_descriptor: Some(CanvasBufferSwapChainDescriptor {
                    surface: &surface,
                    format: desc.color_buffer_format,
                }),
                color_buffer_descriptors: Vec::new(),
                depth_stencil_buffer_format: desc.depth_stencil_buffer_format,
            },
        );
        Self {
            canvas_buffer,
            surface,
            window,
        }
    }

    pub fn color_buffer_format(&self) -> CanvasColorBufferFormat {
        self.canvas_buffer.swap_chain().unwrap().format()
    }

    pub fn depth_stencil_buffer_format(&self) -> Option<CanvasDepthStencilBufferFormat> {
        match &self.canvas_buffer.depth_stencil_buffer() {
            Some(v) => Some(v.format()),
            None => None,
        }
    }

    pub fn update_buffer(&mut self, instance: &Instance) {
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

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw()
    }

    pub fn inner_position(&self) -> Result<window::PhysicalPosition<i32>, NotSupportedError> {
        self.window.inner_position()
    }

    pub fn outer_position(&self) -> Result<window::PhysicalPosition<i32>, NotSupportedError> {
        self.window.outer_position()
    }

    pub fn set_outer_position<P>(&self, position: P)
    where
        P: Into<window::Position>,
    {
        self.window.set_outer_position(position);
    }

    pub fn inner_size(&self) -> window::PhysicalSize<u32> {
        self.window.inner_size()
    }

    pub fn outer_size(&self) -> window::PhysicalSize<u32> {
        self.window.outer_size()
    }

    pub fn set_inner_size<S>(&mut self, instance: &Instance, size: S)
    where
        S: Into<window::Size>,
    {
        self.window.set_inner_size(size);
        self.update_buffer(instance);
    }

    pub fn set_min_inner_size<S>(&mut self, min_size: Option<S>)
    where
        S: Into<window::Size>,
    {
        self.window.set_min_inner_size(min_size);
    }

    pub fn set_max_inner_size<S>(&mut self, max_size: Option<S>)
    where
        S: Into<window::Size>,
    {
        self.window.set_max_inner_size(max_size);
    }

    pub fn set_title(&self, title: &str) {
        self.window.set_title(title)
    }

    pub fn set_visible(&self, visible: bool) {
        self.window.set_visible(visible)
    }

    pub fn set_resizable(&self, resizable: bool) {
        self.window.set_resizable(resizable)
    }

    pub fn set_minimized(&self, minimized: bool) {
        self.window.set_minimized(minimized)
    }

    pub fn set_maximized(&self, maximized: bool) {
        self.window.set_maximized(maximized)
    }

    pub fn set_fullsceen(&self, fullscreen: Option<window::Fullscreen>) {
        self.window.set_fullscreen(fullscreen)
    }

    pub fn fullscreen(&self) -> Option<window::Fullscreen> {
        self.window.fullscreen()
    }

    pub fn set_decorations(&self, decorations: bool) {
        self.window.set_decorations(decorations)
    }

    pub fn set_always_on_top(&self, always_on_top: bool) {
        self.window.set_always_on_top(always_on_top)
    }

    pub fn set_window_icon(&self, window_icon: Option<window::Icon>) {
        self.window.set_window_icon(window_icon)
    }

    pub fn set_ime_position<P>(&self, position: P)
    where
        P: Into<window::Position>,
    {
        self.window.set_ime_position(position)
    }

    pub fn set_cursor_icon(&self, cursor: window::CursorIcon) {
        self.window.set_cursor_icon(cursor)
    }

    pub fn set_cursor_position<P>(&self, position: P) -> Result<(), ExternalError>
    where
        P: Into<window::Position>,
    {
        self.window.set_cursor_position(position)
    }

    pub fn set_cursor_grab(&self, grab: bool) -> Result<(), ExternalError> {
        self.window.set_cursor_grab(grab)
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        self.window.set_cursor_visible(visible)
    }
}

impl Canvas for CanvasWindow {
    fn current_frame(&mut self) -> Result<CanvasFrame, SwapChainError> {
        self.canvas_buffer.current_frame()
    }

    fn canvas_size(&self) -> &CanvasSize {
        self.canvas_buffer.size()
    }

    fn sample_count(&self) -> SampleCount {
        self.canvas_buffer.sample_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InstanceDescriptor;
    use galvanic_assert::{matchers::*, *};
    use roe_app::{event::EventLoopAnyThread, window::WindowBuilder};

    fn create_window(
        size: window::PhysicalSize<u32>,
        desc: &CanvasWindowDescriptor,
    ) -> (CanvasWindow, Instance) {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let event_loop = EventLoop::<()>::new_any_thread();
        let window = WindowBuilder::new()
            .with_inner_size(size)
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        let window = unsafe { CanvasWindow::from_window(&instance, window, desc) };
        (window, instance)
    }

    #[test]
    #[serial_test::serial]
    fn from_window() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let event_loop = EventLoop::<()>::new_any_thread();
        let window = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        let _canvas_window = unsafe {
            CanvasWindow::from_window(&instance, window, &CanvasWindowDescriptor::default())
        };
    }

    #[test]
    #[serial_test::serial]
    fn from_window_and_surface() {
        let event_loop = EventLoop::<()>::new_any_thread();
        let window = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        let (instance, surface) = unsafe {
            Instance::new_with_compatible_window(&InstanceDescriptor::default(), &window).unwrap()
        };
        let _canvas_window = unsafe {
            CanvasWindow::from_window_and_surface(
                &instance,
                window,
                surface,
                &CanvasWindowDescriptor::default(),
            )
        };
    }

    #[test]
    #[serial_test::serial]
    fn multiple_windows_with_generic_instance() {
        let instance = Instance::new(&InstanceDescriptor::default()).unwrap();
        let event_loop = EventLoop::<()>::new_any_thread();
        let window1 = unsafe {
            CanvasWindow::from_window(
                &instance,
                WindowBuilder::new()
                    .with_visible(false)
                    .build(&event_loop)
                    .unwrap(),
                &CanvasWindowDescriptor::default(),
            )
        };
        let window2 = unsafe {
            CanvasWindow::from_window(
                &instance,
                WindowBuilder::new()
                    .with_visible(false)
                    .build(&event_loop)
                    .unwrap(),
                &CanvasWindowDescriptor::default(),
            )
        };
        expect_that!(&window1.id(), not(eq(window2.id())));
    }

    #[test]
    #[serial_test::serial]
    fn multiple_windows_with_compatible_instance() {
        let event_loop = EventLoop::<()>::new_any_thread();
        let window1 = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .unwrap();
        let (instance, surface) = unsafe {
            Instance::new_with_compatible_window(&InstanceDescriptor::default(), &window1).unwrap()
        };
        let window1 = unsafe {
            CanvasWindow::from_window_and_surface(
                &instance,
                window1,
                surface,
                &CanvasWindowDescriptor::default(),
            )
        };
        let window2 = unsafe {
            CanvasWindow::from_window(
                &instance,
                WindowBuilder::new()
                    .with_visible(false)
                    .build(&event_loop)
                    .unwrap(),
                &CanvasWindowDescriptor::default(),
            )
        };
        expect_that!(&window1.id(), not(eq(window2.id())));
    }

    #[test]
    #[serial_test::serial]
    fn canvas_size() {
        let (window, _) = create_window(
            window::PhysicalSize {
                width: 150,
                height: 30,
            },
            &CanvasWindowDescriptor::default(),
        );
        expect_that!(window.canvas_size(), eq(CanvasSize::new(150, 30)));
    }

    #[test]
    #[serial_test::serial]
    fn canvas_size_after_resizing() {
        let (mut window, instance) = create_window(
            window::PhysicalSize {
                width: 150,
                height: 30,
            },
            &CanvasWindowDescriptor::default(),
        );

        window.set_inner_size(
            &instance,
            window::PhysicalSize {
                width: 200,
                height: 60,
            },
        );
        expect_that!(window.canvas_size(), eq(CanvasSize::new(200, 60)));

        // Changing the min or max size doesn't directly influence the window size.
        window.set_min_inner_size(Some(window::PhysicalSize::<u32> {
            width: 250,
            height: 100,
        }));
        expect_that!(window.canvas_size(), eq(CanvasSize::new(200, 60)));

        window.set_max_inner_size(Some(window::PhysicalSize::<u32> {
            width: 180,
            height: 80,
        }));
        expect_that!(window.canvas_size(), eq(CanvasSize::new(200, 60)));
    }

    #[test]
    #[serial_test::serial]
    fn update_buffer() {
        let (mut window, instance) = create_window(
            window::PhysicalSize {
                width: 150,
                height: 30,
            },
            &CanvasWindowDescriptor::default(),
        );
        expect_that!(window.canvas_size(), eq(CanvasSize::new(150, 30)));
        window.window.set_inner_size(window::PhysicalSize::<u32> {
            width: 200,
            height: 100,
        });
        expect_that!(window.canvas_size(), eq(CanvasSize::new(150, 30)));
        window.update_buffer(&instance);
        expect_that!(window.canvas_size(), eq(CanvasSize::new(200, 100)));
    }

    #[test]
    #[serial_test::serial]
    fn default_buffer_parameters() {
        let (mut window, _) = create_window(
            window::PhysicalSize {
                width: 20,
                height: 30,
            },
            &CanvasWindowDescriptor::default(),
        );

        expect_that!(&window.sample_count(), eq(1));
        expect_that!(
            &window.color_buffer_format(),
            eq(CanvasColorBufferFormat::default())
        );
        expect_that!(&window.depth_stencil_buffer_format(), eq(None));

        let frame = window.current_frame().unwrap();
        expect_that!(frame.swap_chain().is_some());
        expect_that!(frame.color_buffers().is_empty());
        expect_that!(frame.depth_stencil_buffer().is_none());

        let swap_chain_ref = frame.swap_chain().unwrap();
        expect_that!(&swap_chain_ref.sample_count(), eq(1));
        expect_that!(
            &swap_chain_ref.format(),
            eq(CanvasColorBufferFormat::default())
        );
    }

    #[test]
    #[serial_test::serial]
    fn multisampled_window() {
        let (mut window, _) = create_window(
            window::PhysicalSize {
                width: 20,
                height: 30,
            },
            &CanvasWindowDescriptor {
                sample_count: 2,
                ..CanvasWindowDescriptor::default()
            },
        );

        expect_that!(&window.sample_count(), eq(2));
        expect_that!(
            &window.color_buffer_format(),
            eq(CanvasColorBufferFormat::default())
        );
        expect_that!(&window.depth_stencil_buffer_format(), eq(None));

        let frame = window.current_frame().unwrap();
        expect_that!(frame.swap_chain().is_some());
        expect_that!(frame.color_buffers().is_empty());
        expect_that!(frame.depth_stencil_buffer().is_none());

        let swap_chain_ref = frame.swap_chain().unwrap();
        expect_that!(&swap_chain_ref.sample_count(), eq(2));
        expect_that!(
            &swap_chain_ref.format(),
            eq(CanvasColorBufferFormat::default())
        );
    }

    #[test]
    #[serial_test::serial]
    fn with_depth_stencil_buffer() {
        let (mut window, _) = create_window(
            window::PhysicalSize {
                width: 20,
                height: 30,
            },
            &CanvasWindowDescriptor {
                depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
                ..CanvasWindowDescriptor::default()
            },
        );

        expect_that!(&window.sample_count(), eq(1));
        expect_that!(
            &window.color_buffer_format(),
            eq(CanvasColorBufferFormat::default())
        );
        expect_that!(
            &window.depth_stencil_buffer_format(),
            eq(Some(CanvasDepthStencilBufferFormat::Depth32Float))
        );

        let frame = window.current_frame().unwrap();
        expect_that!(frame.swap_chain().is_some());
        expect_that!(frame.color_buffers().is_empty());
        expect_that!(frame.depth_stencil_buffer().is_some());

        let swap_chain_ref = frame.swap_chain().unwrap();
        expect_that!(&swap_chain_ref.sample_count(), eq(1));
        expect_that!(
            &swap_chain_ref.format(),
            eq(CanvasColorBufferFormat::default())
        );

        let ds_buffer_ref = frame.depth_stencil_buffer().unwrap();
        expect_that!(&ds_buffer_ref.sample_count(), eq(1));
        expect_that!(
            &ds_buffer_ref.format(),
            eq(CanvasDepthStencilBufferFormat::Depth32Float)
        );
    }

    #[test]
    #[serial_test::serial]
    fn multisampled_with_depth_stencil_buffer() {
        let (mut window, _) = create_window(
            window::PhysicalSize {
                width: 20,
                height: 30,
            },
            &CanvasWindowDescriptor {
                sample_count: 2,
                depth_stencil_buffer_format: Some(CanvasDepthStencilBufferFormat::Depth32Float),
                ..CanvasWindowDescriptor::default()
            },
        );

        expect_that!(&window.sample_count(), eq(2));
        expect_that!(
            &window.color_buffer_format(),
            eq(CanvasColorBufferFormat::default())
        );
        expect_that!(
            &window.depth_stencil_buffer_format(),
            eq(Some(CanvasDepthStencilBufferFormat::Depth32Float))
        );

        let frame = window.current_frame().unwrap();
        expect_that!(frame.swap_chain().is_some());
        expect_that!(frame.color_buffers().is_empty());
        expect_that!(frame.depth_stencil_buffer().is_some());

        let swap_chain_ref = frame.swap_chain().unwrap();
        expect_that!(&swap_chain_ref.sample_count(), eq(2));
        expect_that!(
            &swap_chain_ref.format(),
            eq(CanvasColorBufferFormat::default())
        );

        let ds_buffer_ref = frame.depth_stencil_buffer().unwrap();
        expect_that!(&ds_buffer_ref.sample_count(), eq(2));
        expect_that!(
            &ds_buffer_ref.format(),
            eq(CanvasDepthStencilBufferFormat::Depth32Float)
        );
    }

    #[test]
    #[serial_test::serial]
    fn non_default_color_buffer_format() {
        let (mut window, _) = create_window(
            window::PhysicalSize {
                width: 20,
                height: 30,
            },
            &CanvasWindowDescriptor {
                color_buffer_format: CanvasColorBufferFormat::Rgba8Unorm,
                ..CanvasWindowDescriptor::default()
            },
        );

        expect_that!(&window.sample_count(), eq(1));
        expect_that!(
            &window.color_buffer_format(),
            eq(CanvasColorBufferFormat::Rgba8Unorm)
        );
        expect_that!(&window.depth_stencil_buffer_format(), eq(None));

        let frame = window.current_frame().unwrap();
        expect_that!(frame.swap_chain().is_some());
        expect_that!(frame.color_buffers().is_empty());
        expect_that!(frame.depth_stencil_buffer().is_none());

        let swap_chain_ref = frame.swap_chain().unwrap();
        expect_that!(&swap_chain_ref.sample_count(), eq(1));
        expect_that!(
            &swap_chain_ref.format(),
            eq(CanvasColorBufferFormat::Rgba8Unorm)
        );
    }
}
