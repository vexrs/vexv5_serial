fn main() {
    let vex_ports = vexv5_serial::devices::ports::discover_vex_ports().unwrap();

    println!("{:?}", vex_ports);
}