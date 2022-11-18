use super::Command;

/// Reads in a key-value entry from the brain.
#[derive(Copy, Clone)]
pub struct KVRead<'a> (pub &'a str);

impl<'a> Command for KVRead<'a> {
    type Response = String;

    /// Encodes a request for the value of a key-value store.
    /// The &str in the struct body is used as the key
    fn encode_request(self) -> Vec<u8> {
        // The payload is just the key, but zero terminated
        let mut payload = self.0.as_bytes().to_vec();
        payload.push(0);

        // Encode an extended command of value 0x2e
        super::Extended(0x2e, &payload).encode_request()
    }

    /// Returns the String value of the key requested.
    fn decode_stream<T: std::io::Read>(stream: &mut T, timeout: std::time::Duration) -> Result<Self::Response, crate::errors::DecodeError> {

        // Read in the extended packet
        let packet = super::Extended::decode_stream(stream, timeout)?;

        // If the command id is wrong, then error
        if packet.0 != 0x2e {
            return Err(crate::errors::DecodeError::ExpectedCommand(0x2e, packet.0));
        }

        // The payload of the packet should just be the value of the kv store
        // minus the null-terminator
        // Suffix here is always &[0] so it will always return Some. We can just unwrap
        Ok(String::from_utf8(packet.1.strip_suffix(&[0]).unwrap().to_vec())?)
    }

}



/// Writes a key-value entry to the brain
#[derive(Copy, Clone)]
pub struct KVWrite<'a> (pub &'a str, pub &'a str);

impl<'a>Command for KVWrite<'a> {
    type Response = ();

    /// Requests an update of an entry the key-value store on the brain
    fn encode_request(self) -> Vec<u8> {

        // Convert the value to an array of bytes
        let value = self.1.as_bytes();

        // Certain keys have a maximum size
        let packet_length = {
            usize::min(self.1.len(),{
                if self.0 == "teamnumber" {
                    7
                } else if self.0 == "robotname" {
                    16
                } else {
                    254
                }
            })
        };

        // Trim the value to the maximum size and convert to a vec so we can push the null-terminator
        let mut value = value[..packet_length].to_vec();
        value.push(0); // Null terminator

        // Likewise convert the key and add a null-terminator
        let mut key = self.0.as_bytes().to_vec();
        key.push(00);

        // The payload is just b"{key}{value}"
        // We will use key as the payload
        key.extend(value);

        // Send the extended command
        super::Extended(0x2f, &key).encode_request()
    }

    /// This returns nothing (()), and serves only to verify that the request was recieved.
    /// It will return an error if the request was recieved incorrectly.
    fn decode_stream<T: std::io::Read>(stream: &mut T, timeout: std::time::Duration) -> Result<Self::Response, crate::errors::DecodeError> {
        
        // Decode as an extended packet
        let packet = super::Extended::decode_stream(stream, timeout)?;

        // If the command id is wrong, then error
        if packet.0 != 0x2f {
            return Err(crate::errors::DecodeError::ExpectedCommand(0x2e, packet.0));
        }

        println!("{:?}", packet.1);

        Ok(())
    }
}
