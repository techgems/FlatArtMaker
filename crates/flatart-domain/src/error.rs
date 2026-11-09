/// The single error type every crate maps its own errors into.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
