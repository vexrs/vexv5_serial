use std::io::Read;



fn main() -> anyhow::Result<()>{
    let vex_ports = vexv5_serial::devices::get_socket_info_pairs()?;

    let ports = vexv5_serial::devices::open_device(&vex_ports[0])?;

    let mut device = vexv5_serial::v5::Device::new(ports.0, ports.1);

    loop {
        // Just read 1 byte at a time
        let mut buf = [0u8; 0x40];

        // Ignore CRC errors
        match device.read_serial(&mut buf) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                vexv5_serial::errors::DecodeError::CrcError => Ok(()),
                _ => Err(e),
            }
        }?;

        // Convert buf to a vector
        let buf = buf.to_vec();

        

        // Print the bytes as utf8
        print!("{}", String::from_utf8(buf)?);

        // Flush stdout
        std::io::Write::flush(&mut std::io::stdout())?;

        // Read a string
        let mut s = String::new();
        std::io::Stdin::read_line(&std::io::stdin(), &mut s)?;
        s.strip_suffix("\n").unwrap();

        // Send it over the serial port
        device.write_serial(s.as_bytes())?;
    }
    Ok(())
}