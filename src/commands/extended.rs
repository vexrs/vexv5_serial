//! Implements a structure for encoding and decoding extended commands.


use crate::errors::VexACKType;
use crate::checks::VexExtPacketChecks;

use super::Command;

/// Encodes an Extended command
/// Depended on by all extended commands.
/// 
/// # Members
/// 
/// * `0` - The extended command id
/// * `1` - The payload of the extended command
/// 
/// # Examples
/// No examples are provided here. For implementation details, see a basic command such as `KVRead` to see how this can be used.
#[derive(Copy, Clone)]
pub struct Extended<'a>(pub u8, pub &'a[u8]);

impl<'a> Extended<'a> {
    /// Decodes an extended payload from a stream
    async fn decode_extended<T: crate::io::Read>(stream: &mut T, timeout: std::time::Duration, checks: VexExtPacketChecks) -> Result<ExtendedResponse, crate::errors::DecodeError> {
        // Decode the simple packet
        let packet = super::Simple::decode_stream(stream, timeout).await?;

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
        let command_id = match packet.1.first() {
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

        // Get the final payload value, removing the last two CRC bytes
        let payload = match packet.1.get(2..packet.1.len()-2) {
            Some(v) => v,
            None => return Err(crate::errors::DecodeError::PacketLengthError)
        }.to_vec();

        // Return the response
        Ok(ExtendedResponse(command_id, payload))
    }
}

#[async_trait]
impl<'a> Command for Extended<'a> {
    type Response = ExtendedResponse;

    fn encode_request(self) -> Vec<u8> {
        
        // Create the empty extended packet, with the extended command ID
        let mut packet = vec![self.0];

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

        // Create the simple packet containing the extended packet
        let mut packet =super::Simple(0x56, &packet).encode_request();

        // Now we need to add the CRC.
        // The CRC that the v5 uses is the common CRC_16_XMODEM.
        // This is defined in the lib.rs of this crate as the implementation the crc crate uses.
        let v5crc = crc::Crc::<u16>::new(&crate::VEX_CRC16);

        // Calculate the crc checksum
        let checksum = v5crc.checksum(&packet);

        // And append it to the packet

        // First the upper byte, then the lower byte (big endian)
        packet.push((checksum >> 8) as u8);
        packet.push((checksum & 0xff) as u8);

        // Return the packet
        packet
    }

    async fn decode_stream<T: crate::io::Read>(stream: &mut T, timeout: std::time::Duration) -> Result<Self::Response, crate::errors::DecodeError> {
        // Pass along to decode_extended, assuming that by default we run all checks
        Extended::decode_extended(stream, timeout, VexExtPacketChecks::ALL).await
    }

    
    
}

/// The response returned by an extended command
/// 
/// # Members
/// 
/// * `0` - The command id of the recieved response
/// * `1` - The payload of the recieved response
pub struct ExtendedResponse(pub u8, pub Vec<u8>);