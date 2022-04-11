pub mod vexdevice;
pub use vexdevice::VEXDevice;

use crate::ports::{VEXSerialInfo, VEXSerialClass};

use bitflags::bitflags;
use anyhow::{Result, anyhow};

pub const SERIAL_TIMEOUT_SECONDS: u64 = 50;
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
            _ => Err(anyhow!("Invalid vex product type.")),
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
    PIT = 0x00,
    /// Used when wirelessly uploading data to the V5
    /// Brain
    UPLOAD = 0x01,
}

/// Different possible vex VIDs
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VexVID { // I also have no idea what this is.
    USER = 1,
    SYSTEM = 15,
    RMS = 16, // I believe that robotmesh studio uses this
    PROS = 24, // PROS uses this one
    MW = 32, // IDK what this one is.
}


/// Finds which V5 serial ports to use.
    /// NOTE: This supports either a controller, brain, or both plugged in
    /// Two controllers will work, but whichever controller was plugged in first
    /// will be used. Unplugging and replugging a controller will not cause it to
    /// be considered "second" however. If you wish to switch controllers, unplug both,
    /// plug in the one you want to use and then plug in the other one.
/// This function will prefer a brain over a controller.
pub fn find_ports(known_ports: Vec<VEXSerialInfo>) -> Result<(VEXSerialInfo, Option<VEXSerialInfo>)> {
    // If there are no ports, then error.
    if known_ports.is_empty() {
        return Err(anyhow!("No ports found"));
    }
    // Find the system port
    let system_port = known_ports.iter().find(|port| {
        port.class == VEXSerialClass::System
    }).unwrap_or_else(||{
        // If no system port was found, then find a controller port
        match known_ports.iter().find(|port| {
            port.class == VEXSerialClass::Controller
        }) {
            Some(port) => port,
            None => &known_ports[0],
        }
    });
    // If it is not a system or controller port, then error
    if system_port.class != VEXSerialClass::System && system_port.class != VEXSerialClass::Controller {
        return Err(anyhow!("No system or controller port found"));
    }
    // Find the user port
    let user_port = known_ports.iter().find(|port| {
        port.class == VEXSerialClass::User
    }).cloned();
    Ok((system_port.clone(), user_port))
}