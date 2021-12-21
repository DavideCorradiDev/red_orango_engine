#![allow(dead_code)]

use rand::Rng;

use roe_os::EventLoopClosed;

use roe_math::Vector3;

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
    ImageError(image::error::ImageError),
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
            Self::ImageError(e) => write!(f, "Image load error ({})", e),
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
            Self::ImageError(e) => Some(e),
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

impl From<image::error::ImageError> for ApplicationError {
    fn from(e: image::error::ImageError) -> Self {
        ApplicationError::ImageError(e)
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
            let current_color = Vector3::new(
                self.current_color.r,
                self.current_color.g,
                self.current_color.b,
            );
            let target_color = Vector3::new(
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

pub enum AudioFormat {
    Wav,
    Ogg,
    Unknown,
}

pub fn read_audio_format<P: AsRef<std::path::Path>>(path: P) -> AudioFormat {
    if let Some(extension) = path.as_ref().extension() {
        let extension = extension.to_ascii_lowercase();
        if extension == "wav" {
            return AudioFormat::Wav;
        }
        if extension == "ogg" {
            return AudioFormat::Ogg;
        }
    }
    AudioFormat::Unknown
}

pub fn load_decoder<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<Box<dyn roe_audio::Decoder>, roe_audio::DecoderError> {
    let format = read_audio_format(&path);
    let input = std::io::BufReader::new(std::fs::File::open(&path)?);
    let decoder: Box<dyn roe_audio::Decoder> = match format {
        AudioFormat::Wav => Box::new(roe_audio::WavDecoder::new(input)?),
        AudioFormat::Ogg => Box::new(roe_audio::OggDecoder::new(input)?),
        AudioFormat::Unknown => return Err(roe_audio::DecoderError::Unimplemented),
    };
    Ok(decoder)
}

fn main() {}
