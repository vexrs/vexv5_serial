use crate::ports::{VEXSerialInfo};
use crate::protocol::{V5Protocol, VEXDeviceCommand, VEXExtPacketChecks};
use anyhow::{Result};
use ascii::AsAsciiStr;

use std::cell::RefCell;
use std::rc::Rc;
use std::io::{Read, Write};
use std::{vec};

use super::{V5DeviceVersion, VexProduct, V5ControllerChannel, VexVID};


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
    pub user_port_writer: Option<T>,
    /// The interrior serial buffer.
    serial_buffer: Vec<u8>,
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
            serial_buffer: vec![],
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

    /// Reads in serial data from the system port.
    #[allow(clippy::unused_io_amount)]
    pub fn read_serial(&mut self, n_bytes: usize) -> Result<Vec<u8>> {
        // If the buffer is too small, read in more
        while self.serial_buffer.len() < n_bytes {
            if let Some(w) = &mut self.user_port_writer {
                // Max out at 255 bytes.
                let mut buf = [0x0u8; 0xff];

                // No read exact here, because we do not know how many bytes will be sent.
                w.read(&mut buf)?;
                self.serial_buffer.extend(buf);
            } else {
                let buf = self.read_serial_raw()?;
                self.serial_buffer.extend(buf);
            }
        }

        // Get the data
        let data: Vec<u8> = self.serial_buffer.drain(0..n_bytes).collect();

        Ok(data)
    }

    /// Reads serial data from the system port
    /// Because the system port primarily sends commands,
    /// serial data should be sent as a command.
    fn read_serial_raw(&mut self) -> Result<Vec<u8>> {
        // The way PROS does this is by caching data until a \00 is received.
        // This is because PROS uses COBS to send data. We will be doing the same in another function.
        // The PROS source code also notes that read and write are the same command and
        // that the way that the difference is signaled is by providing the read length as 0xFF
        // and adding aditional data for write, or just specifying the read length for reading.
        // PROS also caps the payload size at 64 bytes, which we will do as well.

        // Borrow the protocol wrapper as mutable
        let mut protocol = self.protocol.borrow_mut();

        // Pack together data to send -- We are reading on an upload channel
        // and will be reading a maximum of 64 bytes.
        let payload: (u8, u8) = (V5ControllerChannel::UPLOAD as u8, 0x40u8);
        let payload = bincode::serialize(&payload)?;
        
        // Send the command, requesting the data
        protocol.send_extended(VEXDeviceCommand::SerialReadWrite, payload)?;

        // Read the response ignoring CRC and length.
        let response = protocol.receive_extended(VEXExtPacketChecks::ACK | VEXExtPacketChecks::CRC)?;
        
        // Return the data
        Ok(response.1)
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




