use super::Command;

/// The structure base of all Simple commands
/// The first u8 is the Simple command ID, the second is the 
/// simple command's payload
pub struct Simple<'a>(pub u8, pub &'a[u8]);

impl<'a> Command for Simple<'a> {
    type Response = SimpleResponse<'a>;

    fn encode_request(self) -> Vec<u8> {
        
        // Create the simple packet with magic number and command type
        let mut packet = vec![0xc9, 0x36, 0xb8, 0x47, self.0];

        // And just append the payload
        packet.extend(self.1);

        // Return the finished simple packet
        packet
    }

    fn decode_response_payload(payload: Vec<u8>) -> Result<Self::Response, crate::errors::DecodeError> {
        todo!()
    }
}

/// The Simple command response contains the Simple command id, and the response payload
pub struct SimpleResponse<'a>(pub u8, pub &'a[u8]);