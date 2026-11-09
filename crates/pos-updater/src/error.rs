use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error de descarga: {0}")]
    Download(String),

    #[error("Error de verificación: {0}")]
    Verification(String),

    #[error("Error de I/O: {0}")]
    Io(String),

    #[error("Error: {0}")]
    Other(String),
}
