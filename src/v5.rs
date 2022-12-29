//! Contains metadata about the V5
use bitflags::bitflags;

/// Enum that represents the channel
/// for the V5 Controller
#[derive(Copy, Clone)]
pub enum V5ControllerChannel {
    /// Used when driving the robot outside of a competition match
    Pit = 0x00,
    /// Used when wirelessly uploading or downloading data to/from the V5
    /// Brain
    Download = 0x01,
}


/// Enum that represents a vex product
/// 
/// # Variants
/// 
/// * `V5Brain` - Represents a V5 Robot Brain
/// * `V5Controller` - Represents a V5 Robot Controller
#[derive(Copy, Clone, Debug)]
pub enum VexProductType {
    V5Brain(V5BrainFlags),
    V5Controller(V5ControllerFlags)
}

impl From<VexProductType> for u8 {
    
    fn from(product: VexProductType) -> u8 {
        match product {
            VexProductType::V5Brain(_) => 0x10,
            VexProductType::V5Controller(_) => 0x11,
        }
    }
}

impl TryFrom<(u8, u8)> for VexProductType {
    type Error = crate::errors::DeviceError;

    fn try_from(value: (u8,u8)) -> Result<VexProductType, Self::Error> {
        match value.0 {
            0x10 => Ok(VexProductType::V5Brain(V5BrainFlags::from_bits(value.1).unwrap_or(V5BrainFlags::NONE))),
            0x11 => Ok(VexProductType::V5Controller(V5ControllerFlags::from_bits(value.1).unwrap_or(V5ControllerFlags::NONE))),
            _ => Err(crate::errors::DeviceError::InvalidDevice),
        }
    }
}

bitflags!{
    /// Configuration flags for the v5 brain
    pub struct V5BrainFlags: u8 {
        const NONE = 0x0;
    }
    /// Configuration flags for the v5 controller
    pub struct V5ControllerFlags: u8 {
        const NONE = 0x0;
        const CONNECTED_CABLE = 0x01; // From testing, this appears to be how it works.
        const CONNECTED_WIRELESS = 0x02;
    }
}


// # File Transfer structures
// These structures are used during file transfers between the brain and the host



/// The function to be performed during the file transfer
///
/// # Variants
/// 
/// * `Upload` - use when writing to a file on the brain
/// * `Download` - use when reading from a file on the brain
#[repr(u8)]
#[derive(Copy, Clone, Debug)]

pub enum FileTransferFunction {
    Upload = 0x01,
    Download = 0x02,
}

/// The target storage device of a file transfer
/// 
/// # Variants
/// 
/// * `Flash` - The flash memory on the robot brain where most program files are stored
/// * `Screen` - The memory accessed when taking a screen capture from the brain.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum FileTransferTarget {
    Flash = 0x01,
    Screen = 0x02,
}

/// The VID of a file transfer
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum FileTransferVID {
    User = 1,
    System = 15,
    RMS = 16,
    PROS = 24,
    MW = 32,
    Other(u8)
}

impl FileTransferVID {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 =>  Self::User,
            15 => Self::System,
            16 => Self::RMS,
            24 => Self::PROS,
            32 => Self::MW,
            a => Self::Other(a)
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            FileTransferVID::User => 1,
            FileTransferVID::System => 15,
            FileTransferVID::RMS => 16,
            FileTransferVID::PROS => 24,
            FileTransferVID::MW => 32,
            FileTransferVID::Other(a) => a,
        }
    }
}

bitflags! {
    /// Options in a file transfer
    pub struct FileTransferOptions: u8 {
        const NONE = 0x0;
        /// Set to overwite the file
        const OVERWRITE = 0b1;
    }

    
}


/// The File type of a file
/// 
/// * `Bin` - Binary files, generally programs
/// * `Ini` - Ini files for program metadata and configuration
/// * `Other` - Any other file type, including custom user types
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum FileTransferType {
    Bin,
    Ini,
    Other([u8; 3])
}

impl FileTransferType {
    pub fn to_bytes(self) -> [u8; 4] {
        match self {
            Self::Bin => *b"bin\0",
            Self::Ini => *b"ini\0",
            Self::Other(t) => [t[0], t[1], t[2], 0u8],
        }
    }

    pub fn from_bytes(v: &[u8; 4]) -> Self {
        match &v {
            [0x62, 0x69, 0x6e, 0x0] => Self::Bin,
            [0x69, 0x6e, 0x69, 0x0] => Self::Ini,
            _ => Self::Other([v[0], v[1], v[2]])
        }
    }
}

/// The action to run when the transfer is complete
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum FileTransferComplete {
    DoNothing = 0,
    RunProgram = 1,
    ShowRunScreen = 2,
}

/// File metadata returned when requesting file metadata by index
#[derive(Copy, Clone, Debug)]
pub struct FileMetadataByIndex {
    pub idx: u8,
    pub file_type: FileTransferType,
    pub length: u32,
    pub addr: u32,
    pub crc: u32,
    pub timestamp: u32,
    pub version: u32,
    pub name: [u8; 24],
}

/// File metadata returned when requesting file metadata by name
#[derive(Copy, Clone, Debug)]
pub struct FileMetadataByName {
    pub linked_vid: FileTransferVID,
    pub file_type: FileTransferType,
    pub length: u32,
    pub addr: u32,
    pub crc: u32,
    pub timestamp: u32,
    pub version: u32,
    pub linked_filename: [u8; 24],
}