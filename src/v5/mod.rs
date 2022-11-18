use std::io::{Read, Write};

pub mod meta;
/// The representation of a V5 device
pub struct Device<S: Read + Write, U: Read+Write> {
    system_port: S,
    user_port: Option<U>,
    read_buffer: Vec<u8>,
}

impl<S: Read + Write, U: Read+Write> Device<S, U> {
    pub fn new(system: S, user: Option<U>) -> Self {
        
        Device {
            system_port: system,
            user_port: user,
            read_buffer: Vec::new(),
        }
    }

    /// Sends a command and recieves its response
    pub fn send_request<C: crate::commands::Command + Copy>(&mut self, command: C) -> Result<C::Response, crate::errors::DecodeError> {
        // Send the command over the system port
        self.send_command(command)?;

        // Wait for the response
        self.response_for::<C>()
    }

    /// Sends a command
    pub fn send_command<C: crate::commands::Command + Copy>(&mut self, command: C) -> Result<(), crate::errors::DecodeError> {

        // Encode the command
        let encoded = command.encode_request();
        
        // Write the command to the serial port
        match self.system_port.write_all(&encoded) {
            Ok(_) => (),
            Err(e) => return Err(crate::errors::DecodeError::IoError(e))
        };

        match self.system_port.flush() {
            Ok(_) => (),
            Err(e) => return Err(crate::errors::DecodeError::IoError(e))
        };

        Ok(())
    }

    /// Recieves a response for a command
    pub fn response_for<C: crate::commands::Command + Copy>(&mut self) -> Result<C::Response, crate::errors::DecodeError> {
        C::decode_stream(&mut self.system_port, std::time::Duration::from_secs(10))
    }

    /// Reads from the user program serial port over the system port
    pub fn read_serial(&mut self, buf: &mut [u8]) -> Result<usize, crate::errors::DecodeError> {
        
        // Form a custom Extended command to read and write from serial.
        // We do the same as PROS, reading 64 bytes and specifying upload channel for some reason
        let payload = vec![meta::V5ControllerChannel::UPLOAD as u8, 0x40];

        // Send the extended command 0x27
        let res = self.send_request(crate::commands::Extended(0x27, &payload))?;

        // Ensure that the response is for the correct command
        if res.0 != 0x27 {
            return Err(crate::errors::DecodeError::ExpectedCommand(0x27, res.0));
        }

        // The response payload should be the data that we read, so copy it into the read buffer
        self.read_buffer.extend(res.1);

        // The amount of data to read into the buf
        let data_len = usize::min(buf.len(), self.read_buffer.len());

        // Get the data from the read buffer
        let data = &self.read_buffer[..data_len];

        // Strip the data from the read buffer
        self.read_buffer.strip_prefix(data).unwrap(); // This should never panic, because data is taken from read_buffer itself

        // Copy the first bytes of the read_buffer into buf, maxing out at the length of buf.
        // We do this so no data is lost
        buf.copy_from_slice(data);

        // Return the length of the data we read
        Ok(data_len)
    }
}

impl<S, U> std::io::Read for Device<S, U>
where S: Read + Write, U: Read + Write {
    
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {

        // If the user port is available, then just read from it
        if let Some(p) = &mut self.user_port {
            p.read(buf)
        } else {
            // If not, then delegate to the read_serial
            match self.read_serial(buf) {
                Ok(v) => Ok(v),
                Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e))
            }
        }
    }
}