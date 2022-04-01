use anyhow::Result;
use anyhow::private::kind::TraitKind;
use rusb::{Device, GlobalContext, DeviceHandle, TransferType};
use std::io::{Write, Read};
use std::time::Duration;

const VEX_V5_BRAIN_PID: u16 = 0x0501;
const VEX_V5_CONTROLLER_PID: u16 = 0x0503;

/// Represents the class of a vex serial port
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VEXSerialClass {
    User,
    System,
    Controller,
}

pub fn discover_vex_ports() -> Result<()> {
    // Get all serial devices
    let available_ports = serialport::available_ports()?;

    // Iterate over all available ports
    for port in available_ports {
        println!("{:?}", port);
    }

    
    Ok(())
}