use lazy_static::lazy_static;

pub use alto::{
    AltoError as BackendError, AsBufferData, Mono, SampleFrame, Source, SourceState, Stereo,
};

// TODO: limit visibility to crate.
lazy_static! {
    pub static ref ALTO: alto::Alto =
        alto::Alto::load_default().expect("Failed to load the audio library");
}
