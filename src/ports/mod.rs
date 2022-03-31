use anyhow::Result;
use rusb::{Device, GlobalContext, DeviceHandle};
use std::io::{Write, Read};

const VEX_V5_BRAIN_PID: u16 = 0x0501;
const VEX_V5_CONTROLLER_PID: u16 = 0x0503;

/// Represents the class of a vex serial port
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VEXSerialClass {
    User,
    System,
    Controller,
}



#[derive(Debug)]
struct VEXSerialInterface {
    device: Device<GlobalContext>,
    interface: u8,
    class: VEXSerialClass
}

impl VEXSerialInterface {
    fn open(&self) -> Result<VEXSerialHandle> {
        Ok(VEXSerialHandle {
            device: self,
            handle: self.device.open()?,
        })
    }
}

pub struct VEXSerialHandle<'a> {
    device: &'a VEXSerialInterface,
    handle: DeviceHandle<GlobalContext>,
}

impl<'a> Write for VEXSerialHandle<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        Ok(0)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

pub fn discover_vex_ports() -> Result<()> {
    let devices = rusb::devices()?;
    let devices = devices.iter();

    // Find all vex ports
    let devices = devices.filter(|device| {
        let dev_desc = match device.device_descriptor() {
            Ok(desc) => desc,
            _ => {
                return false;
            }
        };

        // If it has a vendor ID of 0x2888 and
        // a product ID of 0x0501 then it is a V5.
        // If the product ID is 0x0503, then it is a V5 Controller
        if dev_desc.vendor_id() == 0x2888 && {
            dev_desc.product_id() == VEX_V5_BRAIN_PID || 
            dev_desc.product_id() == VEX_V5_CONTROLLER_PID
        }{
            return true;
        }



        false
    });

    let mut known_serial_interfaces: Vec<VEXSerialInterface> = Vec::new();
    

    for device in devices {
        let device_desc = device.clone().device_descriptor()?;
        println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id());
        
        let handle = device.open()?;
        
        let conf_desc = device.active_config_descriptor()?;
        for interface in conf_desc.interfaces() {
            for d in interface.descriptors() {
                let di = match d.description_string_index() {
                    Some(di) => di,
                    None => {
                        continue;
                    }
                };
                let desc_str = handle.read_string_descriptor_ascii(di)?;
                println!("{}", desc_str);
                println!("{}", d.interface_number());

                // If this is a controller and the first port
                if  device_desc.product_id() == VEX_V5_CONTROLLER_PID && d.interface_number() == 0 {
                    // Then we only want to add the first interface
                    known_serial_interfaces.push(VEXSerialInterface {
                        device: device.clone(),
                        interface: d.interface_number(),
                        class: VEXSerialClass::Controller
                    });
                    continue;
                }

                // If this is a brain
                if device_desc.product_id() == VEX_V5_BRAIN_PID {
                    // If we are a system port
                    if d.interface_number() == 0 {
                        // Add as a system port
                        known_serial_interfaces.push(VEXSerialInterface {
                            device: device.clone(),
                            interface: d.interface_number(),
                            class: VEXSerialClass::System
                        });
                    }
                    // If we are a user port
                    else if d.interface_number() == 2 {
                        // Add as a user port
                        known_serial_interfaces.push(VEXSerialInterface {
                            device: device.clone(),
                            interface: d.interface_number(),
                            class: VEXSerialClass::User
                        });
                    }
                    // Otherwise dont add
                    continue;
                }

            }
        }
        
    }


    // Iterate over the known serial interfaces
    for interface in known_serial_interfaces {
        
    }

    Ok(())
}