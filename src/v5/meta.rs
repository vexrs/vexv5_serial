//! Contains metadata about the V5
use bitflags::bitflags;

/// Enum that represents the channel
/// for the V5 Controller
pub enum V5ControllerChannel {
    /// Used when wirelessly controlling the 
    /// V5 Brain
    PIT = 0x00,
    /// Used when wirelessly uploading data to the V5
    /// Brain
    UPLOAD = 0x01,
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