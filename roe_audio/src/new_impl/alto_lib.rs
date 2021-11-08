use lazy_static::lazy_static;

pub use alto::AltoError as BackendError;

// TODO: limit visibility to crate.
lazy_static! {
    pub static ref ALTO: alto::Alto =
        alto::Alto::load_default().expect("Failed to load the audio library");
}
