fn main() {
    let vex_ports = vexv5_serial::devices::get_socket_info_pairs().unwrap();

    println!("{:?}", vex_ports);

    let device = vexv5_serial::v5::Device::new(vex_ports.0);

    let res = device.send_command(vexv5_serial::commands::SystemKeyValueWrite("teamnumber", b"7122b"));

    println!("{:?}", res)
}