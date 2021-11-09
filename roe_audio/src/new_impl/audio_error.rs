use super::DecoderError;

pub use alto::AltoError as BackendError;

#[derive(Debug)]
pub enum AudioError {
    BackendError(BackendError),
    DecoderError(DecoderError),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BackendError(e) => write!(f, "Backend error ({})", e),
            Self::DecoderError(e) => write!(f, "Decoder error ({})", e)
        }
    }
}

impl std::error::Error for AudioError { fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BackendError(e) => Some(e),
            Self::DecoderError(e) => Some(e),
        }
    }
}

impl From<BackendError> for AudioError {
    fn from(e: BackendError) -> Self {
        Self::BackendError(e)
    }
}

impl From<DecoderError> for AudioError {
    fn from(e: DecoderError) -> Self {
        Self::DecoderError(e)
    }
}
