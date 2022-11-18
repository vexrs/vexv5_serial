//! Crate for interacting with the Vex V5 Robot brain. Not affiliated with Innovation First Inc.
//! 
//! This crate is structured so that each "command" that can be sent to the robot brain has it's own structure associated with it.
//! Each "command" also has it's own response associated with it. Commands are implemented using the `Command` trait,
//! which currently provides a function to encode the implementing structure to a `Vec<u8>` and a function to decode from a Read stream to the implementing structure.
//! 
//! V5 devices do not have to be accessed over a serial port, but helper functions are provided for finding and opening serial ports.
//! Please note that this example may panic and if it succeeds it *will* change the team number on your brain
//! ```rust
//! 
//! // Find all vex devices on the serial ports
//! let vex_ports = vexv5_serial::devices::get_socket_info_pairs().unwrap();
//! 
//! // Get the first device found (panics if there is no device)
//! let port = &vex_ports[0];
//! 
//! // Open a serial port connection
//! let ports = vexv5_serial::devices::open_device(&vex_ports[0]).unwrap();
//! 
//! // Create a Device struct that can communicate over the port
//! let mut device = vexv5_serial::v5::Device::new(ports.0, ports.1);
//! 
//! // Set the team number on the brain
//! let _ = device.send_request(vexv5_serial::commands::KVWrite("teamnumber", "ABCD")).unwrap();
//! 
//! // Get the new team number and print it
//! let res = device.send_request(vexv5_serial::commands::KVRead("teamnumber")).unwrap();
//! 
//! println!("{}", res);
//! 
//! ```










#![feature(arbitrary_enum_discriminant)]



pub mod commands;
pub mod v5;
pub mod errors;
pub mod devices;
pub mod checks;


use crc::Algorithm;

/// Vex uses CRC16/XMODEM as the CRC16.
pub const VEX_CRC16: Algorithm<u16> = crc::CRC_16_XMODEM;