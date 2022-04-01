/*
use crate::ports::{VEXSerialInfo, VEXSerialClass};
use anyhow::{Result, anyhow};
use serialport::{SerialPort, Parity};
use std::{io::{Read, Write}};




pub struct VEXDevice {
    system: VEXSerialInfo,
    system_opened: Box<dyn SerialPort>,
    user: Option<VEXSerialInfo>,
    user_opened: Option<Box<dyn SerialPort>>,
}

/// Implements a low-level interface to a VEX device.
/// Using the read trait will read from the serial ports in this priority order:
/// 1. User port
/// 2. System port
impl VEXDevice {

    /// Creates a new VEXDevice from the given serial ports
    /// NOTE: This supports either a controller, brain, or both plugged in
    /// Two controllers will work, but whichever controller was plugged in first
    /// will be used. Unplugging and replugging a controller will not cause it to
    /// be considered "second" however. If you wish to switch controllers, unplug both,
    /// plug in the one you want to use and then plug in the other one.
    /// This will prefer a brain over a controller.
    pub fn new(known_ports: Vec<VEXSerialInfo>) -> Result<VEXDevice> {
        // If there are no ports, then error.
        if known_ports.len() == 0 {
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

        // Connect to the system port
        let system_port_open = {
            serialport::new(system_port.port_info.port_name.clone(), 115200)
                .parity(serialport::Parity::None)
                .timeout(std::time::Duration::new(5, 0))
                .stop_bits(serialport::StopBits::One).open()?
        };

        // Connect to the user port
        let user_port_open = {
            if let Some(p) = user_port {
                Some(serialport::new(p.port_info.port_name.clone(), 115200)
                .parity(serialport::Parity::None)
                .timeout(std::time::Duration::new(5, 0))
                .stop_bits(serialport::StopBits::One).open()?)
            } else {
                None
            }
        };

        Ok(VEXDevice {
            system: system_port.clone(),
            system_opened: system_port_open,
            user: user_port,
            user_opened: user_port_open,
        })
    }
}
 */