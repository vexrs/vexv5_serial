use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("timedout when waiting for header")]
    HeaderTimeout,
    #[error("expected an extended packet")]
    ExpectedExtended,
    #[error("crc checksum failed")]
    CrcError,
    #[error("packet length is incorrect")]
    PacketLengthError,
    #[error("invalid ack number")]
    InvalidAck,
    #[error("recieved a nack")]
    NACK(VexACKType),
}

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("Serialport Error")]
    SerialportError(#[from] serialport::Error),
}

/// A V5 device can respond with various different acknowledgements.
/// Some, known as NACKs, are errors that the device cannot handle.
/// This list contains all known NACKs as well as ACK.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VexACKType {
    ACK = 0x76,
    NACKCrcError = 0xCE,
    NACKPayloadShort = 0xD0,
    NACKTransferSizeTooLarge = 0xD1,
    NACKProgramCrcFailed = 0xD2,
    NACKProgramFileError = 0xD3,
    NACKUninitializedTransfer = 0xD4,
    NACKInitializationInvalid = 0xD5,
    NACKLengthModFourNzero = 0xD6,
    NACKAddrNoMatch = 0xD7,
    NACKDownloadLengthNoMatch = 0xD8,
    NACKDirectoryNoExist = 0xD9,
    NACKNoFileRoom = 0xDA,
    NACKFileAlreadyExists = 0xDB,
}

impl VexACKType {
    pub fn from_u8(v: u8) -> Result<Self, DecodeError> {
        match v {
            0x76 => Ok(Self::ACK),
            0xCE => Ok(Self::NACKCrcError),
            0xD0 => Ok(Self::NACKPayloadShort),
            0xD1 => Ok(Self::NACKTransferSizeTooLarge),
            0xD2 => Ok(Self::NACKProgramCrcFailed),
            0xD3 => Ok(Self::NACKProgramFileError),
            0xD4 => Ok(Self::NACKUninitializedTransfer),
            0xD5 => Ok(Self::NACKInitializationInvalid),
            0xD6 => Ok(Self::NACKLengthModFourNzero),
            0xD6 => Ok(Self::NACKAddrNoMatch),
            0xD8 => Ok(Self::NACKDownloadLengthNoMatch),
            0xD9 => Ok(Self::NACKDirectoryNoExist),
            0xDA => Ok(Self::NACKNoFileRoom),
            0xDB => Ok(Self::NACKFileAlreadyExists),
            _ => Err(DecodeError::InvalidAck)
        }
    }
}