
use anyhow::Result;

use self::ports::VexSerialInfo;

pub mod ports;

/// Either a pair of user and system serial devices, or a single controller serial device
#[derive(Debug)]
pub enum SocketInfoPairs {
    UserSystem(VexSerialInfo, VexSerialInfo),
    Controller(VexSerialInfo),
    SystemOnly(VexSerialInfo)
}

/// Gets pairs of two user/system ports or one controller port
pub fn get_socket_info_pairs() -> Result<Vec<SocketInfoPairs>> {
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