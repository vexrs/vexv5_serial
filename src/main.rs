
use std::time::Duration;

use anyhow::Result;
use vexv5_serial::*;
use std::io::Read;
use ascii::AsAsciiStr;

fn main() -> Result<()> {
    let p = ports::discover_vex_ports()?;

    let selected = device::find_ports(p)?;

    let system = (selected.0.clone(), serialport::new(selected.0.port_info.port_name, 115200)
        .parity(serialport::Parity::None)
        .timeout(Duration::new(device::SERIAL_TIMEOUT_SECONDS, device::SERIAL_TIMEOUT_NS))
        .stop_bits(serialport::StopBits::One).open()?);

    let user = match selected.1 {
        Some(port) => {
            Some((port.clone(), serialport::new(port.port_info.port_name, 115200)
                .parity(serialport::Parity::None)
                .timeout(Duration::new(device::SERIAL_TIMEOUT_SECONDS, device::SERIAL_TIMEOUT_NS))
                .stop_bits(serialport::StopBits::One).open()?))
        },
        None => None
    };

    let mut d = device::VEXDevice::new(system, user)?;
    
    let info = d.get_device_version();
    println!("{:?}", info);

    // Try to start a program
    d.execute_program_file("slot_2.bin".to_string())?;

    // Loop through, recieving serial data
    loop {
        let mut buf = [0u8; 1];
        let n = d.read_exact(&mut buf)?;
        print!("{}", buf.as_ascii_str()?);
    }

    Ok(())
}
