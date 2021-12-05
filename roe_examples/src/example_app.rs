#![allow(dead_code)]

use rand::Rng;

use roe_os::EventLoopClosed;

use roe_math::geometry3;

use roe_graphics::ColorF32;

pub type ApplicationEvent = ();

#[derive(Debug)]
pub enum ApplicationError {
    WindowCreationFailed(roe_os::OsError),
    InstanceCreationFailed(roe_graphics::InstanceCreationError),
    RenderFrameCreationFailed(roe_graphics::SurfaceError),
    FontCreationFailed(roe_text::FontError),
    AudioError(roe_audio::Error),
    IoError(std::io::Error),
    TextureLoadError(roe_assets::TextureCacheError),
    AudioLoadError(roe_assets::AudioCacheError),
    FontLoadError(roe_assets::FontCacheError),
    CustomEventSendingError,
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WindowCreationFailed(e) => write!(f, "Window creation failed ({})", e),
            Self::InstanceCreationFailed(e) => write!(f, "Instance creation failed ({})", e),
            Self::RenderFrameCreationFailed(e) => write!(f, "Render frame creation failed ({})", e),
            Self::FontCreationFailed(e) => write!(f, "Font creation failed ({})", e),
            Self::AudioError(e) => write!(f, "Audio error ({})", e),
            Self::IoError(e) => write!(f, "I/O error ({})", e),
            Self::TextureLoadError(e) => write!(f, "Texture load error ({})", e),
            Self::AudioLoadError(e) => write!(f, "Audio load error ({})", e),
            Self::FontLoadError(e) => write!(f, "Font load error ({})", e),
            Self::CustomEventSendingError => write!(f, "Failed to send custom event"),
        }
    }
}

impl std::error::Error for ApplicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::WindowCreationFailed(e) => Some(e),
            Self::InstanceCreationFailed(e) => Some(e),
            Self::RenderFrameCreationFailed(e) => Some(e),
            Self::FontCreationFailed(e) => Some(e),
            Self::AudioError(e) => Some(e),
            Self::IoError(e) => Some(e),
            Self::TextureLoadError(e) => Some(e),
            Self::AudioLoadError(e) => Some(e),
            Self::FontLoadError(e) => Some(e),
            Self::CustomEventSendingError => None,
        }
    }
}

impl From<roe_os::OsError> for ApplicationError {
    fn from(e: roe_os::OsError) -> Self {
        ApplicationError::WindowCreationFailed(e)
    }
}

impl From<roe_graphics::InstanceCreationError> for ApplicationError {
    fn from(e: roe_graphics::InstanceCreationError) -> Self {
        ApplicationError::InstanceCreationFailed(e)
    }
}

impl From<roe_graphics::SurfaceError> for ApplicationError {
    fn from(e: roe_graphics::SurfaceError) -> Self {
        ApplicationError::RenderFrameCreationFailed(e)
    }
}

impl From<roe_text::FontError> for ApplicationError {
    fn from(e: roe_text::FontError) -> Self {
        ApplicationError::FontCreationFailed(e)
    }
}

impl From<roe_audio::Error> for ApplicationError {
    fn from(e: roe_audio::Error) -> Self {
        ApplicationError::AudioError(e)
    }
}

impl From<roe_audio::DecoderError> for ApplicationError {
    fn from(e: roe_audio::DecoderError) -> Self {
        Self::from(roe_audio::Error::from(e))
    }
}

impl From<std::io::Error> for ApplicationError {
    fn from(e: std::io::Error) -> Self {
        ApplicationError::IoError(e)
    }
}

impl From<roe_assets::TextureCacheError> for ApplicationError {
    fn from(e: roe_assets::TextureCacheError) -> Self {
        ApplicationError::TextureLoadError(e)
    }
}

impl From<roe_assets::AudioCacheError> for ApplicationError {
    fn from(e: roe_assets::AudioCacheError) -> Self {
        ApplicationError::AudioLoadError(e)
    }
}

impl From<roe_assets::FontCacheError> for ApplicationError {
    fn from(e: roe_assets::FontCacheError) -> Self {
        ApplicationError::FontLoadError(e)
    }
}

impl<T> From<EventLoopClosed<T>> for ApplicationError {
    fn from(_: EventLoopClosed<T>) -> Self {
        ApplicationError::CustomEventSendingError
    }
}

#[derive(Debug)]
pub struct ChangingColor {
    current_color: ColorF32,
    target_color: ColorF32,
}

impl ChangingColor {
    pub fn new(start_color: ColorF32, start_target_color: ColorF32) -> Self {
        Self {
            current_color: start_color,
            target_color: start_target_color,
        }
    }

    pub fn current_color(&self) -> &ColorF32 {
        &self.current_color
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        const COLORS: [ColorF32; 8] = [
            ColorF32::WHITE,
            ColorF32::BLACK,
            ColorF32::RED,
            ColorF32::GREEN,
            ColorF32::BLUE,
            ColorF32::YELLOW,
            ColorF32::CYAN,
            ColorF32::MAGENTA
        ];
        const COLOR_CHANGE_SPEED: f32 = 1.;

        if self.current_color != self.target_color {
            let current_color = geometry3::Point::new(
                self.current_color.r,
                self.current_color.g,
                self.current_color.b,
            );
            let target_color = geometry3::Point::new(
                self.target_color.r,
                self.target_color.g,
                self.target_color.b,
            );
            let next_color = current_color
                + (target_color - current_color).normalize()
                    * COLOR_CHANGE_SPEED
                    * dt.as_secs_f32();

            self.current_color.r = num::clamp(next_color[0], 0., 1.);
            self.current_color.g = num::clamp(next_color[1], 0., 1.);
            self.current_color.b = num::clamp(next_color[2], 0., 1.);
        } else {
            let mut rng = rand::thread_rng();
            self.target_color = COLORS[rng.gen_range(0..COLORS.len())];
        }
    }
}

fn main() {}
