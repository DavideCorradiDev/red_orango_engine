pub use alto::AltoError as BackendError;

#[derive(Debug)]
pub enum Error {
    BackendError(BackendError),
    IoError(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BackendError(e) => write!(f, "Backend error ({})", e),
            Self::IoError(e) => write!(f, "Input output error ({})", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BackendError(e) => Some(e),
            Self::IoError(e) => Some(e),
        }
    }
}
