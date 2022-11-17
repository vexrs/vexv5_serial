use super::Command;

/// The structure base of all Extended commands
/// The first u8 is the extended command ID, the second is the 
/// extended command's payload
pub struct Extended<'a>(pub u8, pub &'a[u8]);

impl<'a> Command for Extended<'a> {
    type Response = ExtendedResponse<'a>;

    fn encode_request(self) -> Vec<u8> {
        
        // Create the empty extended packet
        let mut packet = Vec::<u8>::new();

        // Get the length of the payload
        let payload_length = self.1.len() as u16;

        // If the payload is larger than 0x80, then we need to push the high byte separately
        // This appears to be a primitive varint implementation. We will do what PROS cli
        // does and max out at two bytes
        if payload_length > 0x80 {
            packet.push(((payload_length >> 8) | 0x80) as u8);
        }

        // Push the lower byte
        packet.push((payload_length & 0xff) as u8);

        // Add the payload to the packet
        packet.extend(self.1);

        // Now we need to add the CRC.
        // The CRC that the v5 uses is the common CRC_16_XMODEM.
        // This is defined in the lib.rs of this crate as the implementation the crc crate uses.
        let v5crc = crc::Crc::<u16>::new(&crate::VEX_CRC16);

        // Calculate the crc checksum
        let checksum = v5crc.checksum(&packet);

        // And append it to the packet

        // First the upper byte, then the lower byte (big endian)
        packet.push(((checksum >> 8) & 0xff) as u8);
        packet.push((checksum & 0xff) as u8);

        // Now encode the simple command containing our extended packet and return
        super::Simple(0x56, &packet).encode_request()
    }

    fn decode_response_payload(payload: Vec<u8>) -> Result<Self::Response, crate::errors::DecodeError> {
        todo!()
    }
}

/// The extended command response contains the extended command id, and the response payload
pub struct ExtendedResponse<'a>(pub u8, pub &'a[u8]);