#![allow(dead_code)]

use rand::Rng;

use roe_app::event::EventLoopClosed;

use roe_math::geometry3;

use roe_graphics::ColorF32;

pub type ApplicationEvent = ();

#[derive(Debug)]
pub enum ApplicationError {
    WindowCreationFailed(roe_app::window::OsError),
    InstanceCreationFailed(roe_graphics::InstanceCreationError),
    RenderFrameCreationFailed(roe_graphics::SwapChainError),
    FontCreationFailed(roe_text::FontError),
    AudioError(roe_audio::AudioError),
    IoError(std::io::Error),
    CustomEventSendingError,
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::WindowCreationFailed(e) => {
                write!(f, "Window creation failed ({})", e)
            }
            ApplicationError::InstanceCreationFailed(e) => {
                write!(f, "Instance creation failed ({})", e)
            }
            ApplicationError::RenderFrameCreationFailed(e) => {
                write!(f, "Render frame creation failed ({})", e)
            }
            ApplicationError::FontCreationFailed(e) => {
                write!(f, "Font creation failed ({})", e)
            }
            ApplicationError::AudioError(e) => {
                write!(f, "Audio error ({})", e)
            }
            ApplicationError::IoError(e) => {
                write!(f, "I/O error ({})", e)
            }
            ApplicationError::CustomEventSendingError => {
                write!(f, "Failed to send custom event")
            }
        }
    }
}

impl std::error::Error for ApplicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ApplicationError::WindowCreationFailed(e) => Some(e),
            ApplicationError::InstanceCreationFailed(e) => Some(e),
            ApplicationError::RenderFrameCreationFailed(e) => Some(e),
            ApplicationError::FontCreationFailed(e) => Some(e),
            ApplicationError::AudioError(e) => Some(e),
            ApplicationError::IoError(e) => Some(e),
            ApplicationError::CustomEventSendingError => None,
        }
    }
}

impl From<roe_app::window::OsError> for ApplicationError {
    fn from(e: roe_app::window::OsError) -> Self {
        ApplicationError::WindowCreationFailed(e)
    }
}

impl From<roe_graphics::InstanceCreationError> for ApplicationError {
    fn from(e: roe_graphics::InstanceCreationError) -> Self {
        ApplicationError::InstanceCreationFailed(e)
    }
}

impl From<roe_graphics::SwapChainError> for ApplicationError {
    fn from(e: roe_graphics::SwapChainError) -> Self {
        ApplicationError::RenderFrameCreationFailed(e)
    }
}

impl From<roe_text::FontError> for ApplicationError {
    fn from(e: roe_text::FontError) -> Self {
        ApplicationError::FontCreationFailed(e)
    }
}

impl From<roe_audio::AudioError> for ApplicationError {
    fn from(e: roe_audio::AudioError) -> Self {
        ApplicationError::AudioError(e)
    }
}

impl From<roe_audio::DecoderError> for ApplicationError {
    fn from(e: roe_audio::DecoderError) -> Self {
        Self::from(roe_audio::AudioError::from(e))
    }
}

impl From<std::io::Error> for ApplicationError {
    fn from(e: std::io::Error) -> Self {
        ApplicationError::IoError(e)
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
