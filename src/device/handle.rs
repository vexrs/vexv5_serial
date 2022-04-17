use crate::protocol::{VexDeviceCommand, VexExtPacketChecks};
use anyhow::Result;
use ascii::AsciiString;
use std::cell::RefCell;
use std::rc::Rc;
use std::io::{Read, Write};

use super::{VexInitialFileMetadata, VexFiletransferMetadata, VexFiletransferFinished};






/// This represents a file handle
/// for files on the V5 device.
#[derive(Clone, Debug)]
pub struct V5FileHandle<T> 
    where T: Read + Write {
    pub device: Rc<RefCell<crate::protocol::V5Protocol<T>>>,
    pub transfer_metadata: VexFiletransferMetadata,
    pub metadata: VexInitialFileMetadata,
    pub file_name: AsciiString,
    pub closed: bool,
}

impl<T: Write + Read> V5FileHandle<T> {
    /// Closes the file transfer
    pub fn close(&mut self, on_exit: VexFiletransferFinished) -> Result<Vec<u8>> {


        // Send the exit command
        self.device.borrow_mut().send_extended(VexDeviceCommand::ExitFile, bincode::serialize(&(on_exit as u8))?)?;
        
        // Get the response
        let response = self.device.borrow_mut().receive_extended(VexExtPacketChecks::ALL)?;
        
        self.closed = true;

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
        self.device.borrow_mut().send_extended(VexDeviceCommand::ReadFile, payload)?;

        // Recieve the response
        let response = self.device.borrow_mut().receive_extended(VexExtPacketChecks::CRC)?;
        
        // Truncate to requested data (Ignore the integer sent in the first four bytes)
        let offset = 3;
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
            data.extend(self.read_len(i+self.metadata.addr, (packet_size + 3) & !3)?);
        }

        let data = data[..length as usize].to_vec();
        Ok(data)
    }

    /// Writes a vector of data up to max_packet_size to the file
    /// at the specified offset.
    pub fn write_some(&self, offset: u32, data: Vec<u8>) -> Result<()> {

        // Pad the payload to have a length that is a multiple of four
        let mut data = data;
        data.resize((data.len() + 3) & !3, 0x0);

        // Create the payload
        let mut payload = bincode::serialize(&(offset))?;
        for b in data {
            payload.push(b);
        }
        
        // Send the write command
        let _sent = self.device.borrow_mut().send_extended(VexDeviceCommand::WriteFile, payload)?;
        
        // Recieve and discard the response
        let _response = self.device.borrow_mut().receive_extended(VexExtPacketChecks::ALL)?;
        
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
        if !self.closed {
            self.close(VexFiletransferFinished::DoNothing).unwrap_or_default();
        }
    }
}
