
fn main() -> anyhow::Result<()>{
    let vex_ports = vexv5_serial::devices::get_socket_info_pairs()?;

    let ports = vexv5_serial::devices::open_device(&vex_ports[0])?;

    let mut device = vexv5_serial::v5::Device::new(ports.0, ports.1);



    //device.send_request(vexv5_serial::commands::KVWrite("teamnumber", "123"))?;
    println!("updated teamnumber");


    let v = device.send_request(vexv5_serial::commands::GetSystemVersion())?;

    println!("{:?}", v);

    // Initialize a file transfer
    device.send_request(vexv5_serial::commands::FileTransferInit {
        function: vexv5_serial::v5::meta::FileTransferFunction::Upload,
        target: vexv5_serial::v5::meta::FileTransferTarget::Flash,
        vid: vexv5_serial::v5::meta::FileTransferVID::User,
        options: vexv5_serial::v5::meta::FileTransferOptions::NONE,
        file_type: vexv5_serial::v5::meta::FileTransferType::Other(*b"txt"),
        length: 13,
        addr: 0,
        crc: vexv5_serial::,
        timestamp: 0,
        version: 0,
        name: *b"test.txt\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    })?;

    device.send_request(vexv5_serial::commands::FileTransferWrite(0x0, b"hello, world!"))?;

    // Close the file transfer
    device.send_request(vexv5_serial::commands::FileTransferExit(vexv5_serial::v5::meta::FileTransferComplete::DoNothing))?;


    
    Ok(())
}