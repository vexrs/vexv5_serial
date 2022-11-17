use super::Command;

/// Reads in a key-value entry from the brain.
#[derive(Copy, Clone)]
pub struct KVRead<'a> (pub &'a str);

impl<'a> Command for KVRead<'a> {
    type Response = KVReadResponse<'a>;

    fn encode_request(self) -> Vec<u8> {
        // The payload is just the key, but zero terminated
        let mut payload = self.0.as_bytes().to_vec();
        payload.push(0);

        // Encode an extended command of value 0x2e
        super::Extended(0x2e, &payload).encode_request()
    }

    fn decode_response_payload(payload: Vec<u8>) -> Result<Self::Response, crate::errors::DecodeError> {
        todo!()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct KVReadResponse<'a> (pub &'a [u8]);