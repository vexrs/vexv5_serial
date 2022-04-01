
use anyhow::Result;
use std::io::Read;
use vexv5_serial::*;


fn main() -> Result<()> {
    let p = ports::discover_vex_ports()?;
    
    println!("{:?}", p);

    Ok(())
}
