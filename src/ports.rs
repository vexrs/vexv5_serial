use anyhow::Result;
use serialport::{SerialPortInfo};

const VEX_V5_BRAIN_PID: u16 = 0x0501;
const VEX_V5_CONTROLLER_PID: u16 = 0x0503;
const VEX_VID: u16 = 0x2888;

/// Represents the class of a vex serial port
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VEXSerialClass {
    User,
    System,
    Controller,
}

/// Represents information about a VEX serial port
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VEXSerialInfo {
    pub port_info: SerialPortInfo,
    pub class: VEXSerialClass,
}


pub fn discover_vex_ports() -> Result<Vec<VEXSerialInfo>> {
    // Get all serial devices
    let available_ports = serialport::available_ports()?;

    let mut vex_ports: Vec<VEXSerialInfo> = Vec::new();

    // Iterate over all available ports
    for port in available_ports {
        let port_info = match port.clone().port_type {
            serialport::SerialPortType::UsbPort(info) => info,
            _ => continue,
        };

        // If it does not have the vex vendor id, skip it.
        if port_info.vid != VEX_VID {
            continue;
        }
        // If it is a v5 controller, then add it to the list
        if port_info.pid == VEX_V5_CONTROLLER_PID {
            vex_ports.push(VEXSerialInfo {
                port_info: port.clone(),
                class: VEXSerialClass::Controller,
            });
        }

        // If it is a v5 brain, then add it to the list
        if port_info.pid == VEX_V5_BRAIN_PID {
            
            vex_ports.push(VEXSerialInfo {
                port_info: port.clone(),
                class: {
                    if vex_ports.is_empty() {
                        VEXSerialClass::System // According to PROS code comments, system is always first
                    } else if vex_ports.last().unwrap().class == VEXSerialClass::System{
                        // If the last one was system, then this one is user.
                        VEXSerialClass::User
                    } else {
                        // Otherwise, it is system.
                        VEXSerialClass::System
                    }
                    
                },
            });
        }

    }

    
    Ok(vex_ports)
}