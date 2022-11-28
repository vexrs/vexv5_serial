use crate::v5::meta::{
    FileTransferFunction,
    FileTransferTarget,
    FileTransferVID,
    FileTransferOptions,
    FileTransferType
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
            self.function.into(),
            self.target.into(),
            self.vid.into(),
            self.options.into()
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

        // Convert the file name into a 24 byte long ASCII string
        let file_name = self.file_name.as_ascii_str()?;
        let mut file_name_bytes: [u8; 24] = [0; 24];
        for (i, byte) in file_name.as_slice().iter().enumerate() {
            if i + 1 > 24 {
                break;
            }
            file_name_bytes[i] = *byte as u8;
        }

        // Add the file name to the payload
        payload.extend(file_name_bytes);

        // Encode an extended command with id 0x11
        super::Extended(0x11, payload).encode_request()
    }

    fn decode_response(command_id: u8, data: Vec<u8>) -> Result<Self::Response, crate::errors::DecodeError> {
        // Decode the extended command
        let payload = super::Extended::decode_response(command_id, data)?;

        // Ensure that it is a response to 0x11
        if payload.0 != 0x11 {
            return Err(crate::errors::DecodeError::ExpectedCommand(0x11, payload.0));
        }

        // Get the max_packet_size (bytes 0..1)
        let max_packet_size = u16::from_le_bytes(payload.1.get(0..1)?);

        // Get the file_size (bytes 2..3)
        let file_size = u16::from_le_bytes(payload.1.get(2..3)?);

        // Get the crc (bytes 4..8)
        let crc = u32::from_le_bytes(payload.1.get(4..7));

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