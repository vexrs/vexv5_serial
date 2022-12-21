
fn main() -> anyhow::Result<()>{
    let vex_ports = vexv5_serial::devices::get_socket_info_pairs()?;
    println!("{:?}", vex_ports);
    let ports = vexv5_serial::devices::open_device(&vex_ports[0])?;

    let mut device = vexv5_serial::v5::Device::new(ports.0, ports.1);



    //device.send_request(vexv5_serial::commands::KVWrite("teamnumber", "123"))?;
    println!("updated teamnumber");


    let v = device.send_request(vexv5_serial::commands::GetSystemVersion())?;

    println!("{:?}", v);

    //let v5crc = crc::Crc::<u32>::new(&vexv5_serial::VEX_CRC32);

    // Initialize a file transfer
    device.send_request(vexv5_serial::commands::FileTransferInit {
        function: vexv5_serial::v5::meta::FileTransferFunction::Download,
        target: vexv5_serial::v5::meta::FileTransferTarget::Flash,
        vid: vexv5_serial::v5::meta::FileTransferVID::User,
        options: vexv5_serial::v5::meta::FileTransferOptions::NONE,
        file_type: vexv5_serial::v5::meta::FileTransferType::Other(*b"txt"),
        length: 0,
        addr: 0,
        crc: 0,
        timestamp: 0,
        version: 0,
        name: *b"test.txt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    })?;

    let d = device.send_request(vexv5_serial::commands::FileTransferRead(0x0, 13))?;

    // Close the file transfer
    device.send_request(vexv5_serial::commands::FileTransferExit(vexv5_serial::v5::meta::FileTransferComplete::DoNothing))?;

    let s = String::from_utf8(d)?;

    println!("{}", s);

    
    Ok(())
}