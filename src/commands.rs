#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ExtCommand<'a> {
    UserRead() = 0x27,
    SystemKeyValueRead(&'a str) = 0x2e,
    SystemKeyValueWrite(&'a str, &'a [u8]) = 0x2f,
}

impl<'a> ExtCommand<'a> {
    pub fn encode(self) -> Vec<u8> {
        match self {
            ExtCommand::SystemKeyValueRead(k) => self.read_kv(k),
            ExtCommand::SystemKeyValueWrite(k, v) => self.write_kv(k, v),
            _ => todo!("Not yet implemented")
        }
    }

    /// Generates an extended command that reads the kv store key
    fn read_kv(self, key: &'a str) -> Vec<u8> {
        // Encode the key as a null terminated string
        let mut key_encoded = key.as_bytes().to_vec();
        key_encoded.push(0); // Push the null termination

        // Create the output payload
        let payload = key_encoded; // In this case, it is just the encoded key name

        // Encode the output extended packet
        self.encode_extended(payload)
    }

    /// Generates an extended command that writes the kv store key
    fn write_kv(self, key: &'a str, value: &[u8]) -> Vec<u8> {

        // Get the value length
        let value_length = {
            // Some keys have a smaller max length than 254
            if key == "teamnumber" {
                7
            } else if key == "robotname" {
                16
            } else {
                usize::min(value.len(), 254)
            }
        };

        // Truncate the value to the maximum length
        let mut value = value[..value_length].to_vec();

        // And add a null terminator if there is not already one
        if let Some(v) = value.last() {
            if v != &0 {
                value.push(0);
            }
        } else {
            value.push(0);
        }
        

        // Now create the payload, which is just the null terminated key folowed by the value
        let mut out = key.as_bytes().to_vec();

        // Add the null-terminator
        out.push(0);

        // Add the value
        out.extend(value);

        // Return the output
        out
    }

    /// Encodes the extended data of a packet
    fn encode_extended(self, payload: Vec<u8>) -> Vec<u8> {

        // Create the output vector which starts with the u8 representing the command.
        let mut out: Vec<u8> = vec![self.as_u8()];

        // Get the payload length
        let payload_length = payload.len() as u16; // Truncate the payload to a u16, more can not be sent at once

        // If the payload is longer than 0x80 bytes, then we need to push the upper byte of the u16
        // PROS ORs the payload length with 0x80, which always sets the high bit to true.
        // The same is done here. I assume this is because the v5 uses the high bit being set to
        // represent that the next byte is the lower byte of the same integer, in a primitive
        // 1 OR 2 byte Varint implementation
        if payload_length > 0x80 {
            out.push(((payload_length >> 8) | 0x80) as u8)
        }

        // The lower byte is always pushed
        // We mask off all of the upper bits so that we never get an overflow error
        out.push((payload_length & 0xff) as u8);

        // Just add the payload to the packet. The payload itself depends on the command,
        // but we do not need to handle that here
        out.extend(payload);

        // Now we need to add the CRC.
        // The CRC that the v5 uses is the common CRC_16_XMODEM.
        // This is defined in the lib.rs of this crate as the implementation the crc crate uses.
        let v5crc = crc::Crc::<u16>::new(&crate::VEX_CRC16);

        // Calculate the crc checksum
        let checksum = v5crc.checksum(&out);

        // And append it to the packet

        // First the upper byte, then the lower byte (big endian)
        out.push(((checksum >> 8) & 0xff) as u8);
        out.push((checksum & 0xff) as u8);

        // Return the finished packet
        out
    }
    
    // Gets the u8 value of self
    pub fn as_u8(self) -> u8 {
        match self {
            ExtCommand::UserRead() => 0x27,
            ExtCommand::SystemKeyValueRead(_) => 0x2e,
            ExtCommand::SystemKeyValueWrite(_, _) => 0x2f,
        }
    }
}

#[repr(u8)]
pub enum Command<'a> {
    Extended(ExtCommand<'a>) = 0x56
}

impl<'a> Command<'a> {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![0xc9, 0x36, 0xb8, 0x47];

        out.extend(match self {
            Command::Extended(c) => {
                // Set the packet type to extended
                let mut ret = vec![Command::Extended as u8];
                
                // Extend with the actual payload
                ret.extend(c.encode());

                ret
            }
        });

        out
    }
}