
#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let vex_ports = vexv5_serial::devices::bluetoothv5::scan_for_v5_devices(None).await?;
    println!("{vex_ports:?}");

    let mut port = vex_ports[0].clone();

    port.connect().await?;
    println!("connected");
    port.handshake().await?;
    println!("finished handshake");
    port.disconnect().await?;
    println!("disconnected");
    
    Ok(())
}