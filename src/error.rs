use thiserror::Error;

/// A type alias for [`std::result::Result<T, remux::Error>`].
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    /// An I/O error occurred. This is typically raised by the underlying
    /// transport.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}
