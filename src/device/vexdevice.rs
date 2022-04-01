use crate::ports::{VEXSerialInfo};
use crate::protocol::{V5Protocol, VEXDeviceCommand, VEXExtPacketChecks};
use anyhow::{Result, anyhow};
use ascii::AsAsciiStr;

use std::cell::RefCell;
use std::rc::Rc;
use std::io::{Read, Write};
use std::cmp;

use super::{V5DeviceVersion, VexProduct, V5ControllerFlags, V5ControllerChannel, VexVID};


/// This represents a VEX device connected through a serial port.
pub struct VEXDevice<T>
    where T: Read + Write {
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

impl<T: Read + Write> VEXDevice<T> {
    /// Creates a new VEXDevice from the given serial ports
    pub fn new(system: (VEXSerialInfo, T), user: Option<(VEXSerialInfo, T)>) -> Result<VEXDevice<T>> {
        let u = user.map(|(u, w)| (Some(u), Some(w))).unwrap_or((None, None));

        Ok(VEXDevice {
            port: system.0,
            protocol: Rc::new(RefCell::new(V5Protocol::new(system.1, None))),
            user_port: u.0,
            user_port_writer: u.1,
        })
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


    /// If this is a controller, switches to a given channel
    pub fn switch_channel(&mut self, channel: Option<V5ControllerChannel>) -> Result<()> {

        // If the channel is none, then switch back to pit
        let channel = channel.unwrap_or(V5ControllerChannel::PIT);

        // Send the command
        self.protocol.borrow_mut().send_extended(VEXDeviceCommand::SwitchChannel, Vec::<u8>::from([channel as u8]))?;

        // Recieve and discard the response
        let _response = self.protocol.borrow_mut().receive_extended(VEXExtPacketChecks::ALL)?;

        Ok(())
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

    /// Executes a program file on the v5 brain's flash.
    pub fn execute_program_file(&self, file_name: String) -> Result<()> {

        // Convert the name to ascii
        let file_name = file_name.as_ascii_str()?;
        let mut file_name_bytes: [u8; 24] = [0; 24];
        for (i, byte) in file_name.as_slice().iter().enumerate() {
            if (i + 1) > 24 {
                break;
            }
            file_name_bytes[i] = *byte as u8;
        }

        

        // Create the payload
        let payload: (u8, u8, [u8; 24]) = (VexVID::USER as u8, 0, file_name_bytes);
        let payload = bincode::serialize(&payload)?;

        // Borrow protocol as mut
        let mut protocol = self.protocol.borrow_mut();

        // Send the command
        protocol.send_extended(VEXDeviceCommand::ExecuteFile, payload)?;
        
        // Read the response
        let _response = protocol.receive_extended(VEXExtPacketChecks::ALL)?;

        Ok(())
    }
}



// Reads and writes to the vex device should be done through the user port if it exists,
// alternatively using the system port if it doe snot exist.
impl<T: Read + Write> Read for VEXDevice<T> {
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

impl<T: Read + Write> Write for VEXDevice<T> {
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
