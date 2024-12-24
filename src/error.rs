use thiserror::Error;

#[derive(Error, Debug)]
pub enum LiteMonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("System info error: {0}")]
    SysInfo(String),
    
    #[error("Terminal UI error: {0}")]
    Ui(String),
}

pub type Result<T> = std::result::Result<T, LiteMonError>; 