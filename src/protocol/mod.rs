pub mod v5;
pub use v5::V5Protocol;
use crc::Algorithm;
use bitflags::bitflags;

/// Vex uses CRC16/XMODEM as the CRC16.
pub const VEX_CRC16: Algorithm<u16> = crc::CRC_16_XMODEM;

/// Vex uses a CRC32 that I found on page
/// 6 of this document: 
/// <https://www.matec-conferences.org/articles/matecconf/pdf/2016/11/matecconf_tomsk2016_04001.pdf>
pub const VEX_CRC32: Algorithm<u32> = Algorithm {
    poly: 0x04C11DB7,
    init: 0x00000000,
    refin: false,
    refout: false,
    xorout: 0x00000000,
    check: 0x89A1897F,
    residue: 0x00000000,
};

/// The default timeout should be 0.1 seconds.
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 5;
pub const DEFAULT_TIMEOUT_NS: u32 = 0;


/// There are various commands that can be sent to the V5 device.
/// This enum is used to represent those commands.
/// If you discover new commands, PLEASE make a pull request to add them here,
/// or at least file an issue telling me what they are.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum VexDeviceCommand {
    SwitchChannel = 0x10,
    OpenFile = 0x11,
    ExitFile = 0x12,
    WriteFile = 0x13,
    ReadFile = 0x14,
    SetLinkedFilename = 0x15,
    GetDirectoryCount = 0x16,
    GetMetadataByFileIndex = 0x17,
    ExecuteFile = 0x18,
    GetMetadataByFilename = 0x19,
    SetFileMetadata = 0x1A,
    DeleteFile = 0x1B,
    SerialReadWrite = 0x27,
    Extended = 0x56,
    GetSystemVersion = 0xA4,
}


/// A V5 device can respond with various different acknowledgements.
/// Some, known as NACKs, are errors that the device cannot handle.
/// This list contains all known NACKs as well as ACK.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
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

bitflags! {
    /// These flags determine what checks recieve_extended will perform
    /// on the recieved packet.
    pub struct VexExtPacketChecks: u8 {
        const NONE = 0b00000000;
        const ACK = 0b00000001;
        const CRC = 0b00000010;
        const LENGTH = 0b00000100;
        const ALL = Self::ACK.bits | Self::CRC.bits | Self::LENGTH.bits;
    }
}