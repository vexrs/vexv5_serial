use crate::ports::{VEXSerialInfo};
use crate::protocol::{V5Protocol, VEXDeviceCommand, VEXExtPacketChecks};
use anyhow::{Result};
use ascii::{AsAsciiStr, AsciiString};

use std::cell::RefCell;
use std::rc::Rc;
use std::io::{Read, Write};
use std::{vec};

use super::{V5DeviceVersion, VexProduct, V5ControllerChannel, VexVID, VexInitialFileMetadata, VexFiletransferMetadata, VexFileTarget, VexFileMode, VexFiletransferFinished};


/// This represents a file handle
/// for files on the V5 device.
#[derive(Clone, Debug)]
pub struct V5FileHandle<T> 
    where T: Read + Write {
    device: Rc<RefCell<V5Protocol<T>>>,
    pub transfer_metadata: VexFiletransferMetadata,
    pub metadata: VexInitialFileMetadata,
    pub file_name: AsciiString,
}

impl<T: Write + Read> V5FileHandle<T> {
    /// Closes the file transfer
    pub fn close(&mut self, on_exit: VexFiletransferFinished) -> Result<Vec<u8>> {


        // Send the exit command
        self.device.borrow_mut().send_extended(VEXDeviceCommand::ExitFile, bincode::serialize(&(on_exit as u8))?)?;

        // Get the response
        let response = self.device.borrow_mut().receive_extended(VEXExtPacketChecks::ALL)?;
        
        // Return the response data
        Ok(response.1)
    }

    /// Reads n bytes from the file
    pub fn read_len(&self, offset: u32, n_bytes: u16) -> Result<Vec<u8>> {

        // Pad out the number of bytes to be a multiple of four
        let n_bytes_pad = (n_bytes + 3) & !3;

        // Create a payload containing the offset
        // and the number of bytes to read
        let payload = bincode::serialize(&(offset, n_bytes_pad))?;

        // Send the read command
        self.device.borrow_mut().send_extended(VEXDeviceCommand::ReadFile, payload)?;

        // Recieve the response
        let response = self.device.borrow_mut().receive_extended(VEXExtPacketChecks::CRC)?;

        // Truncate to requested data (Ignore the integer sent in the first four bytes)
        let offset = 4;
        let data = response.1[offset..offset + n_bytes as usize].to_vec();

        Ok(data)
    }

    /// Reads the entire file
    pub fn read_all(&self) -> Result<Vec<u8>> {
        // Create the buffer to store data in
        let mut data = Vec::<u8>::new();

        let max_size: u16 = 512;
        let length = self.transfer_metadata.file_size;

        // Iterate over the file's size in steps of max_packet_size
        for i in (0..length).step_by(max_size.into()) {
            
            // Find the packet size that we want to read in
            let packet_size = if i + <u32>::from(max_size) > length {
                <u16>::try_from(length - i)?
            } else {
                max_size
            };
            
            // Read the data and append it to the buffer
            data.extend(self.read_len(i+self.metadata.addr, packet_size)?);
        }
        Ok(data)
    }

    /// Writes a vector of data up to max_packet_size to the file
    /// at the specified offset.
    fn write_some(&self, offset: u32, data: Vec<u8>) -> Result<()> {

        // Pad the payload to have a length that is a multiple of four
        let mut data = data;
        data.resize((data.len() + 3) & !3, 0x0);

        // Create the payload
        let mut payload = bincode::serialize(&(offset))?;
        for b in data {
            payload.push(b);
        }
        
        // Send the write command
        let _sent = self.device.borrow_mut().send_extended(VEXDeviceCommand::WriteFile, payload)?;
        
        // Recieve and discard the response
        let _response = self.device.borrow_mut().receive_extended(VEXExtPacketChecks::ALL)?;
        
        Ok(())
    }

    /// Writes a vector up to the file length of data to the file. 
    /// Ignores any extra bytes at the end of the vector.
    /// Returns the ammount of data read
    pub fn write_all(&self, data: Vec<u8>) -> Result<usize> {

        // Save the max size so it is easier to access
        // We want it to be 3/4 size so we do not have issues with packet headers
        // going over the max size
        let max_size = self.transfer_metadata.max_packet_size / 
        2 + (self.transfer_metadata.max_packet_size / 4);
        
        // We will be using the length of the file in the metadata
        // that way we do not ever write more data than is expected.
        // However, if the vector is smaller than the file size
        // Then use the vector size.
        let size = if data.len() as u32 > self.transfer_metadata.file_size {
            self.transfer_metadata.file_size
        } else {
            data.len() as u32
        };

        

        // We will be incrementing this variable so we know how much we have written
        let mut how_much: usize = 0;
        
        // Iterate over the file's length in steps of max_size
        // We will be writing each iteration.
        for i in (0..size as usize).step_by(max_size.into()) {
            // Determine the packet size. We do not want to write
            // max_size bytes if we are at the end of the file
            let packet_size = if size < max_size as u32 {
                size as u16
            } else if i as u32 + max_size as u32 > size {
                (size - i as u32) as u16
            } else {
                max_size
            };

            // Cut out packet_size bytes out of the provided buffer
            let payload = data[i..i+packet_size as usize].to_vec();

            // Write the payload to the file
            self.write_some(self.metadata.addr + i as u32, payload)?;

            // Increment how_much by packet data so we know how much we
            // have written to the file
            how_much += packet_size as usize;
        }

        Ok(how_much)
    }
}

impl<T: Write + Read> Drop for V5FileHandle<T> {
    fn drop(&mut self) {
        self.close(VexFiletransferFinished::DoNothing).unwrap_or_default();
    }
}



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
        loop {
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

            if self.serial_buffer.len() >= n_bytes {
                break;
            }
        }

        // Get the data.
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

    /// Open a handle to a file on the v5 brain.
    pub fn open(&mut self, file_name: String, file_metadata: Option<VexInitialFileMetadata>) -> Result<V5FileHandle<T>> {

        // Convert the file name into a 24 byte long ASCII string
        let file_name = file_name.as_ascii_str()?;
        let mut file_name_bytes: [u8; 24] = [0; 24];
        for (i, byte) in file_name.as_slice().iter().enumerate() {
            if i + 1 > 24 {
                break;
            }
            file_name_bytes[i] = *byte as u8;
        }

        // Get the default metadata
        let file_metadata = file_metadata.unwrap_or_default();

        // Get a tuple from the file function
        let ft: (u8, u8, u8) = match file_metadata.function {
            VexFileMode::Upload(t, o) => {
                (1, match t {
                    VexFileTarget::DDR => 0,
                    VexFileTarget::FLASH => 1,
                    VexFileTarget::SCREEN => 2,
                }, o as u8)
            },
            VexFileMode::Download(t, o) => {
                (2, match t {
                    VexFileTarget::DDR => 0,
                    VexFileTarget::FLASH => 1,
                    VexFileTarget::SCREEN => 2,
                }, o as u8)
            }
        };

        // Pack the payload together
        type FileOpenPayload = (
            u8, u8, u8, u8,
            u32, u32, u32,
            [u8; 4],
            u32, u32,
            [u8; 24],
        );
        let payload: FileOpenPayload  = (
            ft.0,
            ft.1,
            file_metadata.vid as u8,
            ft.2 | file_metadata.options,
            file_metadata.length,
            file_metadata.addr,
            file_metadata.crc,
            file_metadata.r#type,
            file_metadata.timestamp,
            file_metadata.version,
            file_name_bytes,
        );
        
        let payload = bincode::serialize(&payload)?;
        
        let mut protocol = self.protocol.borrow_mut();

        // Send the request
        protocol.send_extended(VEXDeviceCommand::OpenFile, payload)?;

        // Receive the response
        let response = protocol.receive_extended(VEXExtPacketChecks::ALL)?;

        // Parse the response
        let response: (u16, u32, u32) = bincode::deserialize(&response.1)?;
        let response = VexFiletransferMetadata {
            max_packet_size: response.0,
            file_size: response.1,
            crc: response.2,
        };

        // Create the file handle
        let handle = V5FileHandle {
            device: Rc::clone(&self.protocol),
            transfer_metadata: response,
            metadata: file_metadata,
            file_name: file_name.to_ascii_string(),
        };

        // Return the handle
        Ok(handle)
    }
}



impl<T: Read+ Write> Read for VEXDevice<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        // Read data if we do nto have enough in the buffer
        if self.serial_buffer.len() < buf.len() {
            let _data = match self.read_serial(0) {
                Ok(d) => d,
                Err(e) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
                }
            };
        }
        

        // Find what length to read
        let len = std::cmp::min(self.serial_buffer.len(), buf.len());

        // Drain it out of the buffer
        let mut data: Vec<u8> = self.serial_buffer.drain(0..len).collect();
        
        // Resize data to be the same size as the buffer
        data.resize(buf.len(), 0x00);

        // Copy the data into the buffer
        buf.copy_from_slice(&data);

        Ok(len)
    }
}