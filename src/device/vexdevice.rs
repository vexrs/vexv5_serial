use crate::ports::{VexSerialInfo};
use crate::protocol::{V5Protocol, VexDeviceCommand, VexExtPacketChecks};
use anyhow::{Result};
use ascii::AsAsciiStr;
use std::cell::RefCell;
use std::rc::Rc;
use std::io::{Read, Write};
use std::{vec};
use super::{V5DeviceVersion, VexProduct, V5ControllerChannel, VexVID, VexInitialFileMetadata, VexFiletransferMetadata, VexFileTarget, VexFileMode, VexFileMetadataByIndex, VexFileMetadataByName, VexFileMetadataSet, VexFiletransferFinished};





/// This represents a Vex device connected through a serial port.
pub struct VexDevice<T>
    where T: Read + Write {
    /// This is the (required) system port that was connected
    /// This will be either a controller or a brain and can be used as a fallback
    /// for generic serial communication.
    pub port: VexSerialInfo,

    /// This is the V5Protocol implementation that wraps the system port.
    protocol: Rc<RefCell<V5Protocol<T>>>,

    /// This is the (optional) user port that was connected
    /// that will be used for generic serial communications.
    pub user_port: Option<VexSerialInfo>,
    user_port_writer: Option<T>,
    /// The interrior serial buffer.
    serial_buffer: Vec<u8>,
}

impl<T: Read + Write> VexDevice<T> {
    /// Creates a new VexDevice from the given serial ports
    pub fn new(system: (VexSerialInfo, T), user: Option<(VexSerialInfo, T)>) -> Result<VexDevice<T>> {
        let u = user.map(|(u, w)| (Some(u), Some(w))).unwrap_or((None, None));

        Ok(VexDevice {
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
        protocol.send_simple(VexDeviceCommand::GetSystemVersion, Vec::new())?;

        let version = protocol.receive_simple()?.1;

        // Parse the version data
        let version = V5DeviceVersion {
            system_version: (version[0], version[1], version[2], version[3], version[4]),
            product_type: VexProduct::try_from((version[5], version[6]))?,
        };

        Ok(version)
    }

    /// Switch the controller channel
    fn switch_channel(&mut self, channel: Option<V5ControllerChannel>) -> Result<()> {
        // If this is not a controller
        let info = self.get_device_version()?;
        if let VexProduct::V5Controller(_) = info.product_type {
            // If the channel is none, then switch back to pit
            let channel = channel.unwrap_or(V5ControllerChannel::PIT);

            // Send the command
            self.protocol.borrow_mut().send_extended(VexDeviceCommand::SwitchChannel, Vec::<u8>::from([channel as u8]))?;

            // Recieve and discard the response
            let _response = self.protocol.borrow_mut().receive_extended(VexExtPacketChecks::ALL)?;

            Ok(())
        } else {
            Ok(())
        }
    }

    /// Acts as a context manager to switch to a different controller channel.
    pub fn with_channel<F>(&mut self, channel: V5ControllerChannel, f: F) -> Result<()>
        where F: Fn() -> Result<()> {
        self.switch_channel(Some(channel))?;
        let res = f();
        self.switch_channel(None)?;
        res
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
    fn read_serial_raw(&self) -> Result<Vec<u8>> {
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
        protocol.send_extended(VexDeviceCommand::SerialReadWrite, payload)?;

        // Read the response ignoring CRC and length.
        let response = protocol.receive_extended(VexExtPacketChecks::ACK | VexExtPacketChecks::CRC)?;
        
        // Return the data
        Ok(response.1)
    }

    /// Executes a program file on the v5 brain's flash.
    pub fn execute_program_file(&self, file_name: String, vid: Option<VexVID>, options: Option<u8>) -> Result<()> {

        let vid = vid.unwrap_or_default();
        let options = options.unwrap_or_default();

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
        let payload: (u8, u8, [u8; 24]) = (vid as u8, options, file_name_bytes);
        let payload = bincode::serialize(&payload)?;

        // Borrow protocol as mut
        let mut protocol = self.protocol.borrow_mut();

        // Send the command
        protocol.send_extended(VexDeviceCommand::ExecuteFile, payload)?;
        
        // Read the response
        let _response = protocol.receive_extended(VexExtPacketChecks::ALL)?;

        Ok(())
    }

    /// Open a handle to a file on the v5 brain.
    pub fn open(&mut self, file_name: String, file_metadata: Option<VexInitialFileMetadata>) -> Result<super::V5FileHandle<T>> {

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
        protocol.send_extended(VexDeviceCommand::OpenFile, payload)?;

        // Receive the response
        let response = protocol.receive_extended(VexExtPacketChecks::ALL)?;

        // Parse the response
        let response: (u16, u32, u32) = bincode::deserialize(&response.1)?;
        let response = VexFiletransferMetadata {
            max_packet_size: response.0,
            file_size: response.1,
            crc: response.2,
        };

        // If this is opening for write, then 
        // set the linked filename
        if let VexFileMode::Upload(_, _) = file_metadata.function {
            // Create the payload
            let payload: (u8, u8, [u8; 24]) = (
                file_metadata.vid as u8,
                file_metadata.options | ft.2,
                file_name_bytes
            );
            let payload = bincode::serialize(&payload)?;
            
            // Send the command
            protocol.send_extended(VexDeviceCommand::SetLinkedFilename, payload)?;
            protocol.receive_extended(VexExtPacketChecks::ALL)?;

        }

        // Create the file handle
        let handle = super::V5FileHandle {
            device: Rc::clone(&self.protocol),
            transfer_metadata: response,
            metadata: file_metadata,
            file_name: file_name.to_ascii_string(),
        };

        // Return the handle
        Ok(handle)
    }

    /// Closes the current file transfer
    fn file_transfer_close(&self, on_exit: Option<VexFiletransferFinished>) -> Result<Vec<u8>> {

        let on_exit = on_exit.unwrap_or(VexFiletransferFinished::DoNothing);

        let mut protocol = self.protocol.borrow_mut();

        // Send the exit command
        protocol.send_extended(VexDeviceCommand::ExitFile, bincode::serialize(&(on_exit as u8))?)?;

        // Get the response
        let response = protocol.receive_extended(VexExtPacketChecks::ALL)?;
        
        // Return the response data
        Ok(response.1)
    }

    /// Gets the metadata of a file from it's index number
    pub fn file_metadata_from_index(&self, index: u8, options: Option<u8>) -> Result<VexFileMetadataByIndex> {

        let options = options.unwrap_or_default();

        // Pack together the payload
        let payload = bincode::serialize(&(index, options))?;

        // Borrow the protocol wrapper
        let mut protocol = self.protocol.borrow_mut();

        // Send the command
        protocol.send_extended(VexDeviceCommand::GetMetadataByFileIndex, payload)?;

        // Recieve the response
        let response = protocol.receive_extended(VexExtPacketChecks::ALL)?;

        // Unpack the data
        let response: VexFileMetadataByIndex = bincode::deserialize(&response.1)?;

        Ok(response)
    }

    /// Gets the metadata of a file from it's name
    pub fn file_metadata_from_name(&self, name: String, vid: Option<VexVID>, options: Option<u8>) -> Result<VexFileMetadataByName> {

        let vid = vid.unwrap_or_default();
        let options = options.unwrap_or_default();

        // Convert the file name into a 24 byte long ASCII string
        let file_name = name.as_ascii_str()?;
        let mut file_name_bytes: [u8; 24] = [0; 24];
        for (i, byte) in file_name.as_slice().iter().enumerate() {
            if i + 1 > 24 {
                break;
            }
            file_name_bytes[i] = *byte as u8;
        }

        // Pack together the payload
        let payload = bincode::serialize(&(vid as u8, options, file_name_bytes))?;

        // Borrow the protocol wrapper
        let mut protocol = self.protocol.borrow_mut();

        // Send the command
        protocol.send_extended(VexDeviceCommand::GetMetadataByFilename, payload)?;

        // Recieve the response
        let response = protocol.receive_extended(VexExtPacketChecks::ALL)?;

        // Unpack the data
        let response: VexFileMetadataByName = bincode::deserialize(&response.1)?;

        Ok(response)
    }

    /// Sets the metadata of a program file
    pub fn set_program_file_metadata(&self, name: String, metadata: VexFileMetadataSet) -> Result<()> {

        // Convert the file name into a 24 byte long ASCII string
        let file_name = name.as_ascii_str()?;
        let mut file_name_bytes: [u8; 24] = [0; 24];
        for (i, byte) in file_name.as_slice().iter().enumerate() {
            if i + 1 > 24 {
                break;
            }
            file_name_bytes[i] = *byte as u8;
        }

        // Pack together the payload
        let payload = bincode::serialize(&(metadata, file_name_bytes))?;

        // Borrow the protocol wrapper
        let mut protocol = self.protocol.borrow_mut();

        // Send the command
        protocol.send_extended(VexDeviceCommand::SetFileMetadata, payload)?;

        // Recieve and discard the response
        let _response = protocol.receive_extended(VexExtPacketChecks::ALL);

        Ok(())
    }

    /// Gets the number of directories on the v5 brain
    pub fn get_directory_count(&self, vid: Option<VexVID>, options: Option<u8>) -> Result<i16> {

        let vid = vid.unwrap_or_default();
        let options = options.unwrap_or_default();

        // Pack together the payload
        let payload = bincode::serialize(&(vid as u8, options))?;

        // Borrow the protocol wrapper as mutable
        let mut protocol = self.protocol.borrow_mut();

        // Request the size
        protocol.send_extended(VexDeviceCommand::GetDirectoryCount, payload)?;
        let response = protocol.receive_extended(VexExtPacketChecks::ALL)?;

        // Unpack the size and return
        Ok(bincode::deserialize(&response.1)?)
    }

    /// Erases a file from V5 flash
    /// If erase all is specified then it will erase all files
    /// matching the base name. This defaults to true
    pub fn delete_file(&self, name: String, vid: Option<VexVID>, erase_all: Option<bool>) -> Result<()> {

        let vid = vid.unwrap_or_default();
        let erase_all = erase_all.unwrap_or(true);


        // Apply the erase all option if needed
        let options: u8 = if erase_all {
            0x00 | 0x80
        } else {
            0x00
        };

        // Convert the file name into a 24 byte long ASCII string
        let file_name = name.as_ascii_str()?;
        let mut file_name_bytes: [u8; 24] = [0; 24];
        for (i, byte) in file_name.as_slice().iter().enumerate() {
            if i + 1 > 24 {
                break;
            }
            file_name_bytes[i] = *byte as u8;
        }

        // Pack and send the payload
        let payload = bincode::serialize(&(vid as u8, options, file_name_bytes))?;
        let mut protocol = self.protocol.borrow_mut();
        protocol.send_extended(VexDeviceCommand::DeleteFile, payload)?;

        // Discard the response
        protocol.receive_extended(VexExtPacketChecks::ALL)?;

        // According to PROS, a file transfer is started here, so we should end it
        self.file_transfer_close(None)?;

        Ok(())
    }
}



impl<T: Read+ Write> Read for VexDevice<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        // Read data if we do not have enough in the buffer
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