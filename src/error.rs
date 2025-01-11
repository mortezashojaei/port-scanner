use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScanError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Service detection error: {0}")]
    ServiceDetection(String),
}
