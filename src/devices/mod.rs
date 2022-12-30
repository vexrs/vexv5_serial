//! Implements functions and structures for interacting with vex devices.

pub mod genericv5;

/// The default timeout for a serial connection in seconds
pub const SERIAL_TIMEOUT_SECONDS: u64 = 30;

/// The default timeout for a serial connection in nanoseconds
pub const SERIAL_TIMEOUT_NS: u32 = 0;

/// The USB PID of the V5 Brain
const VEX_V5_BRAIN_USB_PID: u16 = 0x0501;

/// The USB PID of the V5 Controller
const VEX_V5_CONTROLLER_USB_PID: u16 = 0x0503;

/// The USB VID for Vex devices
const VEX_USB_VID: u16 = 0x2888;

/// This enum represents three types of Vex serial devices:
/// The User port for communication with the user program.
/// The System port for communicating with VexOS.
/// And the Controller port for communicating with the VexV5 joystick
#[derive(PartialEq, Debug, Clone)]
pub enum VexPortType {
    User,
    System,
    Controller,
}

/// The type of a vex device
pub enum VexDeviceType {
    Brain,
    Controller,
    Unknown
}

/// This struct represents generic serial information for a vex device
pub struct VexDevice {
    /// The platform-specific name of the system port
    pub system_port: String,

    /// The platform-specific name of the user port
    pub user_port: Option<String>,
    
    /// The type of the device
    pub device_type: VexDeviceType
}
