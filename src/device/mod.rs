use crate::ports::{VEXSerialInfo, VEXSerialClass};
use crate::protocol::V5Protocol;
use anyhow::{Result, anyhow};

use std::io::{Read, Write};

pub const SERIAL_TIMEOUT_SECONDS: u64 = 3;
pub const SERIAL_TIMEOUT_NS: u32 = 0;


/// This represents a VEX device connected through a serial port.
pub struct VEXDevice<T>
    where T: Read + Write + Clone {
    /// This is the (required) system port that was connected
    /// This will be either a controller or a brain and can be used as a fallback
    /// for generic serial communication.
    pub port: VEXSerialInfo,

    /// This is the V5Protocol implementation that wraps the system port.
    protocol: V5Protocol<T>,

    /// This is the (optional) user port that was connected
    /// that will be used for generic serial communications.
    pub user_port: Option<VEXSerialInfo>,
    user_port_writer: Option<T>,
}

impl<T: Read + Write + Clone> VEXDevice<T> {
    /// Creates a new VEXDevice from the given serial ports
    pub fn new(system: (VEXSerialInfo, T), user: Option<(VEXSerialInfo, T)>) -> Result<VEXDevice<T>> {
        Ok(VEXDevice {
            port: system.0,
            protocol: V5Protocol::new(system.1, None),
            user_port: user.clone().map(|x| x.0),
            user_port_writer: user.map(|x| x.1),
        })
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




}



// Reads and writes to the vex device should be done through the user port if it exists,
// alternatively using the system port if it doe snot exist.
impl<T: Read + Write + Clone> Read for VEXDevice<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        match self.user_port_writer {
            Some(ref mut writer) => writer.read(buf),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "No user port found")),
        }
    }
}

impl<T: Read + Write + Clone> Write for VEXDevice<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        match self.user_port_writer {
            Some(ref mut writer) => writer.write(buf),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "No user port found")),
        }
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        match self.user_port_writer {
            Some(ref mut writer) => writer.flush(),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "No user port found")),
        }
    }
}
