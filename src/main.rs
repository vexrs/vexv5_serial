

fn main() -> anyhow::Result<()>{
    let vex_ports = vexv5_serial::devices::get_socket_info_pairs().unwrap();

    println!("{:?}", vex_ports);

    let ports = vexv5_serial::devices::open_device_pair(vex_ports[0].clone())?;

    let mut device = vexv5_serial::v5::Device::new(vex_ports[0].clone(), ports.0, ports.1);

    let res = device.send_request(vexv5_serial::commands::KVRead("teamnumber"));

    println!("{:?}", res);

    Ok(())
}