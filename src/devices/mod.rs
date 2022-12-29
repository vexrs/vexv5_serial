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

/// This trait implements the requirements for a vex device returned by one of the many find_device functions.
pub trait VexDevice<S: std::io::Read + std::io::Write, U: std::io::Read + std::io::Write> {
    fn get_system_port(&self) -> S;
    fn get_user_port(&self) -> Option<U>;
}
