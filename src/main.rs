mod ports;


use anyhow::Result;

fn main() -> Result<()> {
    let p = ports::discover_vex_ports()?;

    Ok(())
}
