use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
}