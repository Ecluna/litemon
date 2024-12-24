use thiserror::Error;

#[derive(Debug, Error)]
pub enum LiteMonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("No GPU found")]
    NoGpuFound,
    #[error("GPU error: {0}")]
    Gpu(#[from] nvml_wrapper::error::NvmlError),
}

pub type Result<T> = std::result::Result<T, LiteMonError>; 