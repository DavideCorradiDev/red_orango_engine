use std::{path::Path};

pub trait AssetLoader<T>
{
    fn load<P: AsRef<Path>>(&self, path: &P) -> Result<T, AssetLoadError>;
}

#[derive(Debug)]
pub enum AssetLoadError {
    IoError(std::io::Error),
    OtherError(String),
}

impl std::fmt::Display for AssetLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Input / Output error ({})", e),
            Self::OtherError(e) => write!(f, "Asset loading error ({})", e),
        }
    }
}

impl std::error::Error for AssetLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::OtherError(_) => None,
        }
    }
}

impl From<std::io::Error> for AssetLoadError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}