mod ports;

use anyhow::Result;

fn main() -> Result<()> {
    ports::discover_vex_ports()?;

    Ok(())
}
