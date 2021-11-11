use lazy_static::lazy_static;

pub use alto::{ContextAttrs as ContextDesc, Source, SourceState};

lazy_static! {
    pub(crate) static ref ALTO: alto::Alto =
        alto::Alto::load_default().expect("Failed to load the audio library");
}
