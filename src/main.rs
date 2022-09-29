fn main() {
    let vex_ports = vexv5_serial::devices::get_socket_info_pairs().unwrap();

    println!("{:?}", vex_ports);
}