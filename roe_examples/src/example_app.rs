#![allow(dead_code)]

use rand::Rng;

use roe_app::window;

use roe_app::event::EventLoopClosed;

use roe_math::geometry3;

use roe_graphics::{ColorF32, InstanceCreationError, SwapChainError};

use roe_text::FontError;

pub type ApplicationEvent = ();

#[derive(Debug)]
pub enum ApplicationError {
    WindowCreationFailed(window::OsError),
    InstanceCreationFailed(InstanceCreationError),
    RenderFrameCreationFailed(SwapChainError),
    FontCreationFailed(FontError),
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
            ApplicationError::CustomEventSendingError => None,
        }
    }
}

impl From<window::OsError> for ApplicationError {
    fn from(e: window::OsError) -> Self {
        ApplicationError::WindowCreationFailed(e)
    }
}

impl From<InstanceCreationError> for ApplicationError {
    fn from(e: InstanceCreationError) -> Self {
        ApplicationError::InstanceCreationFailed(e)
    }
}

impl From<SwapChainError> for ApplicationError {
    fn from(e: SwapChainError) -> Self {
        ApplicationError::RenderFrameCreationFailed(e)
    }
}

impl From<FontError> for ApplicationError {
    fn from(e: FontError) -> Self {
        ApplicationError::FontCreationFailed(e)
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
            self.target_color = COLORS[rng.gen_range(0, COLORS.len())];
        }
    }
}

fn main() {}
