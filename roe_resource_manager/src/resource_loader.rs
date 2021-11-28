use super::Error;

pub trait ResourceLoader<T> {
    type Resource;
    fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self::Resource, Error>;
}