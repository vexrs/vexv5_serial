#![feature(arbitrary_enum_discriminant)]

pub mod commands;
pub mod v5;
pub mod errors;
pub mod devices;


use crc::Algorithm;

/// Vex uses CRC16/XMODEM as the CRC16.
pub const VEX_CRC16: Algorithm<u16> = crc::CRC_16_XMODEM;