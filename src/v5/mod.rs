use std::io::{Read, Write};

use crate::devices::SocketInfoPairs;

/// The representation of a V5 device
pub struct Device<S: Read + Write, U: Read+Write> {
    wrapped_pair: SocketInfoPairs,
    system_port: S,
    user_port: Option<U>
}

impl<S: Read + Write, U: Read+Write> Device<S, U> {
    pub fn new(wraps: SocketInfoPairs) -> Self {
        // Create the user and system ports
        let (user, system) = match wraps {
            SocketInfoPairs::UserSystem(system, user) => {
                (
                    serialport::new(user.port_info.port_name, 115200)
                    .parity(serialport::Parity::None)
                    .timeout(std::time::Duration::new(crate::devices::SERIAL_TIMEOUT_SECONDS, crate::devices::SERIAL_TIMEOUT_NS))
                    .stop_bits(serialport::StopBits::One).open()?,

                    serialport::new(system.port_info.port_name, 115200)
                    .parity(serialport::Parity::None)
                    .timeout(std::time::Duration::new(crate::devices::SERIAL_TIMEOUT_SECONDS, crate::devices::SERIAL_TIMEOUT_NS))
                    .stop_bits(serialport::StopBits::One).open()?
                )
            },
            SocketInfoPairs::Controller(system) => {
                (
                    None,
                    serialport::new(system.port_info.port_name, 115200)
                    .parity(serialport::Parity::None)
                    .timeout(std::time::Duration::new(crate::devices::SERIAL_TIMEOUT_SECONDS, crate::devices::SERIAL_TIMEOUT_NS))
                    .stop_bits(serialport::StopBits::One).open()?
                )
            },
            SocketInfoPairs::SystemOnly(system) => {
                (
                    None,
                    serialport::new(system.port_info.port_name, 115200)
                    .parity(serialport::Parity::None)
                    .timeout(std::time::Duration::new(crate::devices::SERIAL_TIMEOUT_SECONDS, crate::devices::SERIAL_TIMEOUT_NS))
                    .stop_bits(serialport::StopBits::One).open()?
                )
            },
        };
        Device {
            wrapped_pair: wraps,
            system_port: system,
            user_port: user
        }
    }

    pub fn send_command<T, C: crate::commands::Command>(&mut self, command: C) -> Result<C::Response, crate::errors::DecodeError> {
        // Encode the command
        let encoded = command.encode_request();

        // Send the command over the system port
        self.internal_send_command(encoded)?;

        // Wait for the response
        let res = self.response_for(command)?;

        // Decode the response payload
        C::decode_response_payload(res)
    }

    fn internal_send_command(&mut self, encoded: Vec<u8>) -> Result<(), crate::errors::DecodeError> {
        match self.system_port.write_all(&encoded) {
            Ok(v) => Ok(v),
            Err(e) => Err(crate::errors::DecodeError::IoError(e))
        }
    }
}