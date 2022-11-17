use crate::checks::VexExtPacketChecks;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ExtResponse {
    UserRead() = 0x27,
    SystemKeyValueRead() = 0x2e,
    SystemKeyValueWrite() = 0x2f,
}

impl ExtResponse {
    /// Decodes an extended response from the payload
    /// Checks is a bitflag of various packet checks we should perform.
    pub fn decode(data: Vec<u8>, checks: VexExtPacketChecks) -> Result<ExtResponse> {
        // If we should check CRC, then do so
        if checks.contains(VexExtPacketChecks::CRC) {
            // Use the CRC_16_XMODEM CRC that the V5 uses
            let v5crc = crc::Crc::new(&crate::VEX_CRC16);

            // Run the checksum
            if v5crc.checksum(&data) != 0 {
                // Return a failure result
            }
        }

        todo!()
    }

}

#[repr(u8)]
pub enum Response {
    Extended(ExtResponse) = 0x56
}

impl Response {
    /// This function decodes a response packet based on the packet command and payload
    /// Getting this information from a serial stream requires extra logic.
    /// If you are using a Read stream, see decode_stream
    /// The checks argument dictates what checks we should perform on the recieved packet.
    /// This is just passed on to ExtResponse
    pub fn decode(command: u8, payload: Vec<u8>, checks: VexExtPacketChecks) -> Response {
        // If it is an extended command, then delegate to ExtResponse
        // Any other command is not supported for now
        if command == 0x56 {
            Response::Extended(ExtResponse::decode(payload), checks)
        } else {
            panic!("vecv5_serial does not support any commands other then extended");
        }
    }
}