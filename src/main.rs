

fn main() -> anyhow::Result<()>{
    let vex_ports = vexv5_serial::devices::get_socket_info_pairs().unwrap();

    let ports = vexv5_serial::devices::open_device(vex_ports[0].clone())?;

    let mut device = vexv5_serial::v5::Device::new(vex_ports[0].clone(), ports.0, ports.1);

    let res = device.send_request(vexv5_serial::commands::KVWrite("teamnumber", "7122A"));

    let res = device.send_request(vexv5_serial::commands::KVRead("teamnumber")).unwrap();

    println!("{:?}", res);

    Ok(())
}