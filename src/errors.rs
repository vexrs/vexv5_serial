use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("Serialport Error")]
    SerialportError(#[from] serialport::Error),
}