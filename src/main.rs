use std::io::{Read, Write};



fn main() -> anyhow::Result<()>{
    let vex_ports = vexv5_serial::devices::get_socket_info_pairs()?;

    let ports = vexv5_serial::devices::open_device(&vex_ports[0])?;

    let mut device = vexv5_serial::v5::Device::new(ports.0, ports.1);

    loop {
        // Read a string
        let mut s = String::new();
        std::io::Stdin::read_line(&std::io::stdin(), &mut s)?;
        s.strip_suffix("\n").unwrap();

        // Send it over the serial port
        device.write(s.as_bytes())?;
        
        let mut buf = [0u8; 0x40];
        device.read(&mut buf)?;

        // Convert buf to a vector
        let buf = buf.to_vec();

        

        // Print the bytes as utf8
        print!("{}", String::from_utf8(buf)?);

        // Flush stdout
        std::io::Write::flush(&mut std::io::stdout())?;

        
    }
    Ok(())
}