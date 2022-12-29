
fn main() -> anyhow::Result<()>{
    let vex_ports = vexv5_serial::devices::genericv5::find_generic_devices()?;
    println!("{:?}", vex_ports);

    

    let mut device = vex_ports[0].open()?;


    Ok(())
}