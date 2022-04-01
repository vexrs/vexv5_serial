pub mod vexdevice;
pub use vexdevice::VEXDevice;


use bitflags::bitflags;
use anyhow::{Result, anyhow};

pub const SERIAL_TIMEOUT_SECONDS: u64 = 3;
pub const SERIAL_TIMEOUT_NS: u32 = 0;


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

/// This enum is a convenient representation
/// of which type of product the VEX device is.
#[derive(Debug, Clone, Copy)]
pub enum VexProduct {
    V5Brain(V5BrainFlags),
    V5Controller(V5ControllerFlags),
}

impl From<VexProduct> for u8 {
    
    fn from(product: VexProduct) -> u8 {
        match product {
            VexProduct::V5Brain(_) => 0x10,
            VexProduct::V5Controller(_) => 0x11,
        }
    }
}

impl TryFrom<(u8, u8)> for VexProduct {
    type Error = anyhow::Error;

    fn try_from(value: (u8,u8)) -> Result<VexProduct> {
        match value.0 {
            0x10 => Ok(VexProduct::V5Brain(V5BrainFlags::from_bits(value.1).unwrap_or(V5BrainFlags::NONE))),
            0x11 => Ok(VexProduct::V5Controller(V5ControllerFlags::from_bits(value.1).unwrap_or(V5ControllerFlags::NONE))),
            _ => Err(anyhow!("Invalid product type")),
        }
    }
}


/// This struct represents the version of a vex v5 device
#[derive(Debug, Clone, Copy)]
pub struct V5DeviceVersion {
    pub system_version: (u8, u8, u8, u8, u8),
    pub product_type: VexProduct,
}


/// Enum that represents the channel
/// for the V5 Controller
pub enum V5ControllerChannel {
    /// Used when wirelessly controlling the 
    /// V5 Brain
    PIT,
    /// Used when wirelessly uploading data to the V5
    /// Brain
    UPLOAD,
}



