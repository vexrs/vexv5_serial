use serialport::SerialPortInfo;
use anyhow::Result;


/// This enum represents three types of Vex serial devices:
/// The User port for communication with the user program.
/// The System port for communicating with VexOS.
/// And the Controller port for communicating with the VexV5 joystick
pub enum VexSerialType {
    User,
    System,
    Controller,
}

/// This structure incapsulates the information for a vex v5 serial port.
pub struct VexSerialInfo {
    pub port_info: SerialPortInfo,
    pub port_type: VexSerialType,
}

/// This function finds all serial ports that are from VEX devices
/// returning a vector of VexSerialInfo structs/
pub fn discover_vex_ports() -> Result<Vec<VexSerialInfo>> {

    // Get all available serial ports
    let ports = serialport::available_ports()?;

    // Create a vector of all vex ports
    let mut vex_ports: Vec<VexSerialInfo> = Vec::new();

    // Iterate over all available ports
    for port in ports {

        // Get the serial port's info as long as it is a usb port.
        // Other than bluetooth, how would it be possible to have a non-USB
        // serial port. Bluetooth can be handled in a different function
        let port_info = match port.clone().port_type {
            serialport::SerialPortType::UsbPort(info) => info,
            _ => continue, // Skip the port if it is not USB.
        };

    }

    // Return the vector of discovered ports
    Ok(vex_ports)
};
