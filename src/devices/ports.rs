use serialport::SerialPortInfo;
use anyhow::Result;

const VEX_V5_BRAIN_PID: u16 = 0x0501;
const VEX_V5_CONTROLLER_PID: u16 = 0x0503;
const VEX_VID: u16 = 0x2888;

/// This enum represents three types of Vex serial devices:
/// The User port for communication with the user program.
/// The System port for communicating with VexOS.
/// And the Controller port for communicating with the VexV5 joystick
#[derive(PartialEq, Debug)]
pub enum VexSerialType {
    User,
    System,
    Controller,
}

/// This structure incapsulates the information for a vex v5 serial port.
#[derive(Debug)]
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

        // If it does not have a Vex Vendor ID, then skip it
        if port_info.vid != VEX_VID {
            continue;
        }

        // If the Product ID is a Vex Joystick, add it
        if port_info.pid == VEX_V5_CONTROLLER_PID {
            vex_ports.push(VexSerialInfo {
                port_info: port,
                port_type: VexSerialType::Controller,
            });
        } else if port_info.pid == VEX_V5_BRAIN_PID {
            // If it is a brain, add it to the list determining if it is system or user
            vex_ports.push(VexSerialInfo {
                port_info: port,
                port_type: {
                    // Get the product name
                    let name = match port_info.product {
                        Some(s) => s,
                        _ => continue,
                    };

                    // If the name contains User, it is a User port
                    if name.contains("User"){
                        VexSerialType::System
                    } else if name.contains("Communications") {
                        // If the name contains Communications, is is a System port.
                        VexSerialType::User
                    } else if match vex_ports.last() {
                            Some(p) => p.port_type == VexSerialType::System,
                            _ => false,
                        } {
                        // PROS source code also hints that User will always be listed after System
                        VexSerialType::User
                    } else {
                        // If the previous one was user or the vector is empty,
                        // The PROS source code says that this one is most likely System.
                        VexSerialType::System
                    }
                    
                }
            });

        }

    }

    // Return the vector of discovered ports
    Ok(vex_ports)
}
