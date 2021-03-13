extern crate winit;

pub use winit::{dpi::*, error::*, window::*};

#[cfg(target_os = "windows")]
pub use winit::platform::windows::WindowBuilderExtWindows as WindowBuilderExt;

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub use winit::platform::unix::WindowBuilderExtUnix as WindowBuilderExt;

#[cfg(target_os = "macos")]
pub use winit::platform::macos::WindowBuilderExtMacos as WindowBuilderExt;

#[cfg(target_os = "android")]
pub use winit::platform::android::WindowBuilderExtAndroid as WindowBuilderExt;

#[cfg(target_os = "ios")]
pub use winit::platform::ios::WindowBuilderExtIos as WindowBuilderExt;

#[cfg(target_arch = "wasm32")]
pub use winit::platform::web::WindowBuilderExtWeb as WindowBuilderExt;
