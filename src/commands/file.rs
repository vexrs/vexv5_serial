use crate::v5::meta::{
    FileTransferFunction,
    FileTransferTarget,
    FileTransferVID,
    FileTransferOptions,
    FileTransferType, FileTransferComplete
};

use super::Command;


/// Initializes a file transfer between the brain and host
#[derive(Copy, Clone)]
pub struct FileTransferInit {
    function: FileTransferFunction,
    target: FileTransferTarget,
    vid: FileTransferVID,
    options: FileTransferOptions,
    file_type: FileTransferType,
    length: u32,
    addr: u32,
    crc: u32,
    timestamp: u32,
    version: u32,
    name: [u8; 24]
}

impl Command for FileTransferInit {
    type Response = FileTransferInitResponse;

    fn encode_request(self) -> Result<(u8, Vec<u8>), crate::errors::DecodeError> {
        
        // Create the empty payload
        let mut payload = Vec::<u8>::new();

        // Load the function, target, vid, and options
        payload.extend([
            self.function as u8,
            self.target as u8,
            self.vid as u8,
            self.options.bits(),
        ]);

        // Add the length
        payload.extend(self.length.to_le_bytes());

        // Add the addr
        payload.extend(self.addr.to_le_bytes());

        // Add the crc
        payload.extend(self.crc.to_le_bytes());

        // Add the type
        payload.extend(self.file_type.to_bytes());

        // Add the timestamp
        payload.extend(self.timestamp.to_le_bytes());

        // Add the version
        payload.extend(self.version.to_le_bytes());

        // Add the file name to the payload
        payload.extend(self.name);

        // Encode an extended command with id 0x11
        super::Extended(0x11, &payload).encode_request()
    }

    fn decode_response(command_id: u8, data: Vec<u8>) -> Result<Self::Response, crate::errors::DecodeError> {
        // Decode the extended command
        let payload = super::Extended::decode_response(command_id, data)?;

        // Ensure that it is a response to 0x11
        if payload.0 != 0x11 {
            return Err(crate::errors::DecodeError::ExpectedCommand(0x11, payload.0));
        }

        // Get the max_packet_size (bytes 0..1)
        // We can unwrap the try_into because we know that get will return 2 bytes
        let max_packet_size = u16::from_le_bytes(payload.1.get(0..1).ok_or(crate::errors::DecodeError::PacketLengthError)?.try_into().unwrap());

        // Get the file_size (bytes 2..3)
        let file_size = u16::from_le_bytes(payload.1.get(2..3).ok_or(crate::errors::DecodeError::PacketLengthError)?.try_into().unwrap());

        // Get the crc (bytes 4..8)
        let crc = u32::from_le_bytes(payload.1.get(4..7).ok_or(crate::errors::DecodeError::PacketLengthError)?.try_into().unwrap());

        // Return the result
        Ok(FileTransferInitResponse {
            max_packet_size,
            file_size,
            crc
        })
    }
}

#[derive(Copy, Clone)]
pub struct FileTransferInitResponse {
    max_packet_size: u16,
    file_size: u16,
    crc: u32
}



/// Exit a file transfer between the brain and host
/// 
/// # Members
/// 
/// * `0` - The action to complete when the transfer is finished
#[derive(Copy, Clone)]
pub struct FileTransferExit(FileTransferComplete);

impl Command for FileTransferExit {
    type Response = ();

    fn encode_request(self) -> Result<(u8, Vec<u8>), crate::errors::DecodeError> {
        
        // Create the empty payload
        let mut payload = Vec::<u8>::new();

        // Add the file transfer complete byte
        payload.push(self.0 as u8);

        // Encode an extended command with id 0x12
        super::Extended(0x12, &payload).encode_request()
    }

    

    fn decode_response(command_id: u8, data: Vec<u8>) -> Result<Self::Response, crate::errors::DecodeError> {
        
        // Decode the extended command
        let payload = super::Extended::decode_response(command_id, data)?;

        // Ensure that it is a response to 0x12
        if payload.0 != 0x12 {
            return Err(crate::errors::DecodeError::ExpectedCommand(0x11, payload.0));
        }

        // Do nothing
        Ok(())
    }
}