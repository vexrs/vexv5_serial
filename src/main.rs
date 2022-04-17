
use std::{time::Duration, io::Read};

use anyhow::{Result};
use vexv5_serial::{*, device::VexInitialFileMetadata, device::{VexFileMetadataByName, VexVID}};
use ascii::AsAsciiStr;

fn main() -> Result<()> {
    let p = ports::discover_vex_ports()?;

    let selected = device::find_ports(p)?;

    let system = (selected.0.clone(), serialport::new(selected.0.port_info.port_name, 115200)
        .parity(serialport::Parity::None)
        .timeout(Duration::new(device::SERIAL_TIMEOUT_SECONDS, device::SERIAL_TIMEOUT_NS))
        .stop_bits(serialport::StopBits::One).open()?);

    let user = match selected.1 {
        Some(port) => {
            Some((port.clone(), serialport::new(port.port_info.port_name, 115200)
                .parity(serialport::Parity::None)
                .timeout(Duration::new(device::SERIAL_TIMEOUT_SECONDS, device::SERIAL_TIMEOUT_NS))
                .data_bits(serialport::DataBits::Eight)
                .stop_bits(serialport::StopBits::One).open()?))
        },
        None => None
    };

    let mut d = device::VexDevice::new(system, user)?;
    
    let info = d.get_device_version();
    println!("{:?}", info);

    d.with_channel(device::V5ControllerChannel::UPLOAD, |d| {
        // Get the info of slot_1.ini
        let metadata = d.file_metadata_from_name("slot_1.ini".to_string(), None, None)?;
        
        // Read in the slot_1.ini file on the brain
        let mut tdata = VexInitialFileMetadata::default();
        tdata.addr = metadata.addr;
        tdata.vid = VexVID::USER;
        let mut fh = d.open("slot_1.ini".to_string(), Some(VexInitialFileMetadata {
            function: device::VexFileMode::Download(device::VexFileTarget::FLASH, false),
            vid: num::FromPrimitive::from_u8(metadata.linked_vid).unwrap_or(VexVID::USER),
            options: 0,
            length: 0,
            addr: metadata.addr,
            crc: 0,
            r#type: *b"bin\0",
            timestamp: 0,
            version: 0x01000000,
            linked_name: None,
        }))?;


        // Read in data
        let data = fh.read_all()?;

        // Save to file
        std::fs::write("slot_1.ini", data)?;
        
        // Close file
        fh.close(device::VexFiletransferFinished::ShowRunScreen)?;
        
        Ok(())
    })?;


    Ok(())
}
