use crate::ports::{VEXSerialInfo, VEXSerialClass};
use crate::protocol::{V5Protocol, VEXDeviceCommand, VEXExtPacketChecks};
use anyhow::{Result, anyhow};

use std::cell::RefCell;
use std::rc::Rc;
use std::io::{Read, Write};
use std::cmp;

use super::{V5DeviceVersion, VexProduct, V5ControllerFlags, V5ControllerChannel};


/// This represents a VEX device connected through a serial port.
pub struct VEXDevice<T>
    where T: Read + Write + Clone {
    /// This is the (required) system port that was connected
    /// This will be either a controller or a brain and can be used as a fallback
    /// for generic serial communication.
    pub port: VEXSerialInfo,

    /// This is the V5Protocol implementation that wraps the system port.
    protocol: Rc<RefCell<V5Protocol<T>>>,

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
            protocol: Rc::new(RefCell::new(V5Protocol::new(system.1, None))),
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

    /// Retrieves the version of the device
    pub fn get_device_version(&self) -> Result<V5DeviceVersion> {

        // Borrow the protocol as mutable
        let mut protocol = self.protocol.borrow_mut();

        // Request the system information
        protocol.send_simple(VEXDeviceCommand::GetSystemVersion, Vec::new())?;

        let version = protocol.receive_simple()?.1;

        // Parse the version data
        let version = V5DeviceVersion {
            system_version: (version[0], version[1], version[2], version[3], version[4]),
            product_type: VexProduct::try_from((version[5], version[6]))?,
        };

        Ok(version)
    }


    /// Reads serial data from the system port
    /// Because the system port primarily sends commands,
    /// serial data should be sent as a command.
    /// This function should only work if the system port is a controller that is connected wirelessly.
    fn read_serial_remote(&self, buf: &mut [u8]) -> Result<usize> {
        // The way PROS does this is by caching data until a \00 is received.
        // This is because PROS uses COBS to send data. However, because this is a generic library
        // that is not locked to PROS, we will just read data as is.
        // The PROS source code also notes that read and write are the same command and
        // that the way that the difference is signaled is by providing the read length as 0xFF
        // and adding aditional data for write, or just specifying the read length for reading.
        // PROS also caps the payload size at 64 bytes, which we will do as well.

        // Check if we are using a controller that is connected either wirelessly or wired.
        // This is a limit that we put on that will be removed after further testing.
        let can_continue = match self.get_device_version()?.product_type {
            VexProduct::V5Controller(flags) => flags.contains(V5ControllerFlags::CONNECTED_WIRELESS) ||
                flags.contains(V5ControllerFlags::CONNECTED_CABLE),
            _ => false,
        };
        
        if !can_continue {
            return Err(anyhow!("Can only read serial data from a controller that is connected to a brain"));
        }

        // Borrow the protocol wrapper as mutable
        let mut protocol = self.protocol.borrow_mut();


        // Pack together data to send -- We are reading on an upload channel
        // and will be reading a maximum of 64 bytes.
        let payload = bincode::serialize(&(V5ControllerChannel::UPLOAD as u8, 0x40))?;

        // Send the command, requesting the data
        protocol.send_extended(VEXDeviceCommand::SerialReadWrite, payload)?;

        // Read the response
        let response = protocol.receive_extended(VEXExtPacketChecks::ACK | VEXExtPacketChecks::CRC)?;
        
        // Write the response into the buffer
        buf.copy_from_slice(&response.1[0..cmp::min(buf.len(), response.1.len())]);

        // Return the number of bytes read
        Ok(cmp::min(buf.len(), response.1.len()))
    }


}



// Reads and writes to the vex device should be done through the user port if it exists,
// alternatively using the system port if it doe snot exist.
impl<T: Read + Write + Clone> Read for VEXDevice<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        match self.user_port_writer {
            Some(ref mut writer) => writer.read(buf),
            None => {
                // Then read from system port
                match self.read_serial_remote(buf) {
                    Ok(n) => Ok(n),
                    Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
                }
            },
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
            None => {
                // Then there is nothing to flush
                Ok(())
            },
        }
    }
}
