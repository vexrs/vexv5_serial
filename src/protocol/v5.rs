use std::{io::{Read, Write}, time::{Duration, SystemTime}};
use anyhow::{Result, anyhow};
use super::{DEFAULT_TIMEOUT_SECONDS, DEFAULT_TIMEOUT_NS, VEXDeviceCommand, VEXExtPacketChecks, VEXACKType};

/// Wraps an object with Read + Write traits implemented
/// to provide an implementation of the VEX V5 Protocol.
pub struct V5Protocol<T>
    where T: Read + Write {
    /// The read/write object to wrap
    /// This can be a file, serial port, socket, or anything else.
    /// This struct does not care.
    wraps: T,
    timeout: Duration,
}

impl<T> V5Protocol<T>
    where T: Read + Write {
    
    /// Creates a new V5Protocol object
    pub fn new(wraps: T, timeout: Option<Duration>) -> Self {
        V5Protocol {
            wraps,
            timeout: timeout.unwrap_or_else(||{Duration::new(DEFAULT_TIMEOUT_SECONDS, DEFAULT_TIMEOUT_NS)}),
        }
    }

    /// Flushes the write buffer.
    pub fn flush(&mut self) -> Result<()> {
        self.wraps.flush()?;
        Ok(())
    }

    /// Create a simple packet header.
    fn create_simple_packet(&self, command: VEXDeviceCommand) -> Vec<u8> {
        // Just pack together the command and the magic number
        vec![0xc9, 0x36, 0xb8, 0x47, command as u8]
    }

    /// Creates an extended packet.
    /// This function, unlike the create_simple_packet function
    /// includes various other features such as length, CRC, etc.
    fn create_extended_packet(&self, command: VEXDeviceCommand, payload: Vec<u8>) -> Result<Vec<u8>> {

        // Create the packet with the header and command.
        let mut packet: Vec<u8> = vec![0xc9, 0x36, 0xb8, 0x47, VEXDeviceCommand::Extended as u8, command as u8];

        // Get the payload length as a u16;
        let payload_length = payload.len() as u16;

        // If the payload_length is larger than 0x80, then we need to push the upper byte first
        if payload_length > 0x80 {
            packet.push(((payload_length >> 8) | 0x80) as u8);
        }
        // Push the lower byte
        packet.push((payload_length & 0xff) as u8);

        // Add the payload to the packet
        packet.extend(payload);

        // Now calculate the CRC16 of the packet
        let crc = crc::Crc::<u16>::new(&super::VEX_CRC16);
        let crc_result = crc.checksum(&packet);

        // Add the upper byte of the CRC to the packet
        packet.push((crc_result >> 8) as u8);
        // Add the lower byte of the CRC to the packet
        packet.push((crc_result & 0xff) as u8);

        Ok(packet)
    }
    

    /// Revieves a simple packet from the vex device.
    pub fn receive_simple(&mut self) -> Result<(VEXDeviceCommand, Vec<u8>, Vec<u8>)> {
        // We need to wait to recieve the header of a packet.
        // The header should be the bytes [0xAA, 0x55]

        // This header needs to be recieved within the timeout.
        // If it is not recieved within the timeout, then we need to return an error.
        // Begin the countdown now:
        let countdown = SystemTime::now() + self.timeout;

        // Create a buffer for the header bytes
        // This is configurable just in case vex changes the header bytes on us.
        let expected_header: [u8; 2] = [0xAA, 0x55];
        let mut header_index = 0; // This represents what index in the header we will be checking next.

        // The way this works is we recieve a byte from the device.
        // If it matches the current byte (expected_header[header_index]), then we increment the header_index.
        // If the header_index is equal to the length of the header, then we know we have recieved the header.
        // If the header_index is not equal to the length of the header, then we need to keep recieving bytes until we have recieved the header.
        // If an unexpected byte is recieved, reset header_index to zero.
        while header_index < expected_header.len() {
            // If the timeout has elapsed, then we need to return an error.
            // We need to do this first just in case we actually do recieve the header
            // before the timeout has elapsed.
            if countdown < SystemTime::now() {
                return Err(anyhow!("Timeout elapsed while waiting for header."));
            }

            // Recieve a single bytes
            let mut b: [u8; 1] = [0];
            self.wraps.read_exact(&mut b)?;
            let b = b[0];

            if b == expected_header[header_index] {
                header_index += 1;
            } else {
                header_index = 0;
            }
        }

        
        // Now that we know we have recieved the header, we need to recieve the rest of the packet.

        // First create a vector containing the entirety of the recieved packet
        let mut packet: Vec<u8> = Vec::from(expected_header);

        // Read int he next two bytes
        let mut b: [u8; 2] = [0; 2];
        self.wraps.read_exact(&mut b)?;
        packet.extend_from_slice(&b);

        // Get the command byte and the length byte of the packet
        let command = b[0];
        
        // We may need to modify the length of the packet if it is an extended command
        // Extended commands use a u16 instead of a u8 for the length.
        let length = if VEXDeviceCommand::Extended as u8 == command && b[1] & 0x80 == 0x80 {
            // Read the lower bytes
            let mut bl: [u8; 1] = [0];
            self.wraps.read_exact(&mut bl)?;
            packet.push(bl[0]);

            (((b[1] & 0x7f) as u16) << 8) | (bl[0] as u16)
        } else {
            b[1] as u16
        };

        // Read the rest of the payload
        let mut payload: Vec<u8> = vec![0; length as usize];
        // DO NOT CHANGE THIS TO READ. read_exact is required to suppress
        // CRC errors and missing data.
        self.wraps.read_exact(&mut payload)?;
        packet.extend(&payload);

        // Try to convert the u8 representation of the command into
        // a VEXDeviceCommand enum member.
        // If it fails, we do not recognize the command and either the packet is malformed,
        // the device is not a v5 device, or we need to add a new command.
        let command: VEXDeviceCommand = match num::FromPrimitive::from_u8(command) {
            Some(c) => c,
            None => return Err(anyhow!("Unknown command recieved: {}", command)),
        };

        // Now return the data
        // We return the command, the actual payload itself, and the entire packet as a whole.
        Ok((command, payload, packet))
    }

    /// Sends a simple packet to the device. This does not encode the length of the data
    /// because the length depends on the command. Like other write commands, this returns
    /// the number of bytes written.
    pub fn send_simple(&mut self, command: VEXDeviceCommand, data: Vec<u8>) -> Result<usize> {

        // Create the header
        let mut packet = self.create_simple_packet(command);

        // Append the data to the packet
        packet.extend(data);

        // Write the data and flush the buffer
        self.wraps.write_all(&packet)?;
        self.wraps.flush()?;


        Ok(packet.len())
    }

    /// This receives an extended packet from the vex device.
    /// Depending on the flags passed, this will also check the CRC16 of the packet, the
    /// length of the packet and the ACK recieved.
    pub fn receive_extended(&mut self, should_check: VEXExtPacketChecks) -> Result<(VEXDeviceCommand, Vec<u8>, Vec<u8>)> {
        
        // Recieve the underlying simple packet
        let data = self.receive_simple()?;

        // Verify that this is an extended command
        if data.0 != VEXDeviceCommand::Extended {
            return Err(anyhow!("Unexpected command recieved. Expected Extended, got {:?}", data.0));
        }

        // If we are supposed to check the CRC, then do so
        if should_check.contains(VEXExtPacketChecks::CRC) {
            let crc = crc::Crc::<u16>::new(&super::VEX_CRC16);
            if crc.checksum(&data.2) != 0 {
                return Err(anyhow!("CRC16 failed on response."));
            }
        }
        
        // Verify that it is a valid vex command
        let command: VEXDeviceCommand = match num::FromPrimitive::from_u8(data.1[0]) {
            Some(c) => c,
            None => return Err(anyhow!("Unknown command recieved: {}", data.2[2])),
        };

        // Remove the command from the message
        let message = data.1[1..].to_vec();

        // If we should check the ACK, then do so
        if should_check.contains(VEXExtPacketChecks::ACK) {
            // Try to convert the ACK byte into an ACK enum member
            // If it fails, we do not recognize the ACK and either the packet is malformed,
            // the device is not a v5 device, or we need to add a new ACK.
            let ack: VEXACKType = match num::FromPrimitive::from_u8(message[0]) {
                Some(c) => c,
                None => return Err(anyhow!("Unknown ACK recieved: 0x{:x}", message[0])),
            };

            // If it is not an ack, then we need to return an error
            if ack != VEXACKType::ACK {
                return Err(anyhow!("Device NACKED with code {:?}", ack));
            }
        }

        // Get the payload without the ACK byte or the CRC16
        let payload = Vec::from(&message[1..message.len()-2]);
        Ok((command, payload, data.2))
    }

    /// This function sends an extended packet to the vex device.
    /// Like other write commands, this returns the number of bytes written.
    pub fn send_extended(&mut self, command: VEXDeviceCommand, data: Vec<u8>) -> Result<usize> {
        
        // Create the extended packet
        let packet = self.create_extended_packet(command, data)?;

        // Send the packet
        self.wraps.write_all(&packet)?;

        // Flush the buffer
        self.wraps.flush()?;

        Ok(packet.len())
    }
}