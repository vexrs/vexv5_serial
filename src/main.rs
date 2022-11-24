
fn main() -> anyhow::Result<()>{
    let vex_ports = vexv5_serial::devices::get_socket_info_pairs()?;

    let ports = vexv5_serial::devices::open_device(&vex_ports[0])?;

    let mut device = vexv5_serial::v5::Device::new(ports.0, ports.1);

    device.send_request(vexv5_serial::commands::KVWrite("teamnumber", "123"))?;

    Ok(())
}