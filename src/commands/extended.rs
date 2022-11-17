use crate::errors::VexACKType;
use crate::checks::VexExtPacketChecks;

use super::Command;

/// The structure base of all Extended commands
/// The first u8 is the extended command ID, the second is the 
/// extended command's payload
pub struct Extended<'a>(pub u8, pub &'a[u8]);

impl<'a> Extended<'a> {
    /// Decodes an extended payload from a stream
    fn decode_extended<T: std::io::Read>(stream: &mut T, timeout: std::time::Duration, checks: VexExtPacketChecks) -> Result<ExtendedResponse, crate::errors::DecodeError> {
        // Decode the simple packet
        let packet = super::Simple::decode_stream(stream, timeout)?;

        // Ensure that it is an extended packet
        if packet.0 != 0x56 {
            return Err(crate::errors::DecodeError::ExpectedExtended);
        }

        // Check the CRC if we are supposed to
        if checks.contains(VexExtPacketChecks::CRC) {
            // Create the CRC_16_XMODEM CRC that vex uses
            let v5crc = crc::Crc::<u16>::new(&crate::VEX_CRC16);

            // If the checksum on the packet fails, then return an error
            if v5crc.checksum(&packet.2) != 0{
                return Err(crate::errors::DecodeError::CrcError)
            }
        }

        // Get the command id
        let command_id = match packet.1.get(0) {
            Some(v) => *v,
            None => return Err(crate::errors::DecodeError::PacketLengthError)
        };

        // If we should check the ACK, then do so
        if checks.contains(VexExtPacketChecks::ACK) {
            // Get the ack
            let ack = VexACKType::from_u8(match packet.1.get(1) {
                Some(v) => *v,
                None => return Err(crate::errors::DecodeError::PacketLengthError)
            })?;

            // If it is a nack, then fail
            if ack != VexACKType::ACK {
                return Err(crate::errors::DecodeError::NACK(ack));
            }
        }

        // Get the final payload value
        let payload = match packet.1.get(2..) {
            Some(v) => v,
            None => return Err(crate::errors::DecodeError::PacketLengthError)
        }.to_vec();

        // Return the response
        Ok(ExtendedResponse(command_id, payload))
    }
}

impl<'a> Command for Extended<'a> {
    type Response = ExtendedResponse;

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

    fn decode_stream<T: std::io::Read>(stream: &mut T, timeout: std::time::Duration) -> Result<Self::Response, crate::errors::DecodeError> {
        // Pass along to decode_extended, assuming that by default we run all checks
        Extended::decode_extended(stream, timeout, VexExtPacketChecks::ALL)
    }

    
    
}

/// The extended command response contains the extended command id, and the response payload
pub struct ExtendedResponse(pub u8, pub Vec<u8>);