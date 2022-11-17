/// Packet decoding checks
use bitflags::bitflags;

bitflags! {
    /// These flags determine what checks recieve_extended will perform
    /// on the recieved packet.
    pub struct VexExtPacketChecks: u8 {
        const NONE = 0b00000000;
        const ACK = 0b00000001;
        const CRC = 0b00000010;
        const LENGTH = 0b00000100;
        const ALL = Self::ACK.bits | Self::CRC.bits | Self::LENGTH.bits;
    }
}
