
use anyhow::Result;
use serialport::SerialPort;

use self::ports::VexSerialInfo;

pub mod ports;

/// The default timeout for a serial connection in seconds
pub const SERIAL_TIMEOUT_SECONDS: u64 = 30;

/// The default timeout for a serial connection in nanoseconds
pub const SERIAL_TIMEOUT_NS: u32 = 0;


/// Represents the serial ports available for use when connecting to a specific V5 device.
#[derive(Debug, Clone)]
pub enum SocketInfoPairs {
    /// Used when a robot brain is connected.
    /// 
    /// # Members
    /// 
    /// * `0` - The system port that communicates with VEXOS. Commands are sent over this port.
    /// * `1` - The user port that communicates directly with the user program
    /// 
    UserSystem(VexSerialInfo, VexSerialInfo),

    /// Used when a V5 controller is connected to the computer
    /// 
    /// # Members
    /// 
    /// * `0` - The system port of the controller. Commands can be sent over this port.
    /// 
    Controller(VexSerialInfo),

    /// Used when a robot brain is connected but the user port is not available. Almost never used.
    /// 
    /// # Members
    /// 
    /// * `0` - The system port of the brain.
    SystemOnly(VexSerialInfo)
}

/// Discovers all serial ports on the computer that are connected to a V5 and retrieves information about them.
/// 
/// Returns a `Vec` of `SocketInfoPairs`.
pub fn get_socket_info_pairs() -> Result<Vec<SocketInfoPairs>, crate::errors::DeviceError> {
    // Initialize an empty list of pairs
    let mut pairs: Vec<SocketInfoPairs> = Vec::new();

    // Get all vex ports
    let vex_ports = ports::discover_vex_ports()?;

    // Manually iterate over the vex ports
    let mut port_iter = vex_ports.iter().peekable();
    loop {
        // Get the next port in the iteration
        let current_port = match port_iter.next() {
            Some(p) => p,
            None => break,
        };


        if current_port.port_type == ports::VexSerialType::System {
            // Peek the next port, and if it is a User port, add the next pair
            if match port_iter.peek() {
                Some(p) => p.port_type == ports::VexSerialType::User,
                _ => false,
            } {
                pairs.push(SocketInfoPairs::UserSystem(current_port.clone(), match port_iter.next() {
                    Some(p) => p.clone(),
                    None => break,
                }));
                break;
            } else {
                // If not, add a System only port
                pairs.push(SocketInfoPairs::SystemOnly(current_port.clone()));
                break;
            }
        } else if current_port.port_type == ports::VexSerialType::Controller {
            // Add a controlle ronly port
            pairs.push(SocketInfoPairs::Controller(current_port.clone()));
        } else {
            continue;
        }


    }

    Ok(pairs)
}


/// Opens the serial ports for a Vex V5 device.
/// 
/// # Returns
/// 
/// * `0` - The opened system port of either a controller or a brain
/// * `1` - An optional user port that connects to the brain
pub fn open_device(wraps: &SocketInfoPairs) -> Result<(Box<dyn SerialPort>, Option<Box<dyn SerialPort>>), crate::errors::DeviceError> {
    // Create the user and system ports
    Ok(match wraps {
        SocketInfoPairs::UserSystem(system, user) => {
            (
                match serialport::new(&system.port_info.port_name, 115200)
                .parity(serialport::Parity::None)
                .timeout(std::time::Duration::new(crate::devices::SERIAL_TIMEOUT_SECONDS, crate::devices::SERIAL_TIMEOUT_NS))
                .stop_bits(serialport::StopBits::One).open() {
                    Ok(v) => Ok(v),
                    Err(e) => Err(crate::errors::DeviceError::SerialportError(e)),
                }?,
                Some(match serialport::new(&user.port_info.port_name, 115200)
                .parity(serialport::Parity::None)
                .timeout(std::time::Duration::new(crate::devices::SERIAL_TIMEOUT_SECONDS, crate::devices::SERIAL_TIMEOUT_NS))
                .stop_bits(serialport::StopBits::One).open() {
                    Ok(v) => Ok(v),
                    Err(e) => Err(crate::errors::DeviceError::SerialportError(e)),
                }?)
            )
        },
        SocketInfoPairs::Controller(system) => {
            (
                match serialport::new(&system.port_info.port_name, 115200)
                .parity(serialport::Parity::None)
                .timeout(std::time::Duration::new(crate::devices::SERIAL_TIMEOUT_SECONDS, crate::devices::SERIAL_TIMEOUT_NS))
                .stop_bits(serialport::StopBits::One).open() {
                    Ok(v) => Ok(v),
                    Err(e) => Err(crate::errors::DeviceError::SerialportError(e)),
                }?,
                None
            )
        },
        SocketInfoPairs::SystemOnly(system) => {
            (
                match serialport::new(&system.port_info.port_name, 115200)
                .parity(serialport::Parity::None)
                .timeout(std::time::Duration::new(crate::devices::SERIAL_TIMEOUT_SECONDS, crate::devices::SERIAL_TIMEOUT_NS))
                .stop_bits(serialport::StopBits::One).open() {
                    Ok(v) => Ok(v),
                    Err(e) => Err(crate::errors::DeviceError::SerialportError(e)),
                }?,
                None
            )
        },
    })
}