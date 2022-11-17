use super::Command;

/// Reads in a key-value entry from the brain.
#[derive(Copy, Clone)]
pub struct KVRead<'a> (pub &'a str);

impl<'a> Command for KVRead<'a> {
    type Response = KVReadResponse;

    fn encode_request(self) -> Vec<u8> {
        // The payload is just the key, but zero terminated
        let mut payload = self.0.as_bytes().to_vec();
        payload.push(0);

        // Encode an extended command of value 0x2e
        super::Extended(0x2e, &payload).encode_request()
    }

    fn decode_stream<T: std::io::Read>(stream: &mut T, timeout: std::time::Duration) -> Result<Self::Response, crate::errors::DecodeError> {

        // Read in the extended packet
        let packet = super::Extended::decode_stream(stream, timeout)?;

        // The payload of the packet should just be the value of the kv store
        Ok(KVReadResponse(packet.1))
    }

}

#[derive(Clone, Debug)]
pub struct KVReadResponse (pub Vec<u8>);