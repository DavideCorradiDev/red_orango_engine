extern crate winit;

#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopExtWindows as EventLoopExt;

#[cfg(target_os = "linux")]
use winit::platform::unix::EventLoopExtUnix as EventLoopExt;

pub trait EventLoopAnyThread<T: 'static> {
    fn new_any_thread() -> winit::event_loop::EventLoop<T>;
}

impl<T> EventLoopAnyThread<T> for winit::event_loop::EventLoop<T>
where
    T: 'static,
{
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    fn new_any_thread() -> Self {
        <Self as EventLoopExt>::new_any_thread()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    fn new_any_thread() -> Self {
        winit::event_loop::EventLoop::<T>::with_user_event()
    }
}
