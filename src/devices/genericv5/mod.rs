//! Implements discovering, opening, and interacting with vex devices connected over USB. This module does not have async support.
//! Please note that every serial device uses the same structures.


pub mod device;


use crate::devices::VexSerialDeviceInfo;

/// Finds all generic vex v5 devices connected to the computer over usb.
pub fn find_generic_devices() -> Result<Vec<VexSerialDeviceInfo>, crate::errors::DeviceError> {

}