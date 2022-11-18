use std::io::{Read, Write};

pub mod meta;
/// The representation of a V5 device
pub struct Device<S: Read + Write, U: Read+Write> {
    system_port: S,
    user_port: Option<U>,
    read_buffer: Vec<u8>,
    user_read_size: u8,
}

impl<S: Read + Write, U: Read+Write> Device<S, U> {
    pub fn new(system: S, user: Option<U>) -> Self {
        
        Device {
            system_port: system,
            user_port: user,
            read_buffer: Vec::new(),
            user_read_size: 0x20, // By default, read chunks of 32 bytes
        }
    }

    /// Updates the size of the chunks to read from the system port when a user port is not available
    pub fn update_user_read_size(&mut self, user_read_size: u8) {
        self.user_read_size = user_read_size;
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
        
        // Optimization: Only read more bytes from the brain if we need them. This allows usages
        // that use small reads to be much faster.
        if self.read_buffer.len() < buf.len() {
            // Form a custom Extended command to read and write from serial.
            // We do the same as PROS, reading 64 bytes and specifying upload channel for some reason
            // Except we only read up to 64 bytes at a time, so that the user can configure if they want to 
            // read smaller chunks (and thus bypass CRC errors from packet corruption, at the expense of speed)
            let payload = vec![meta::V5ControllerChannel::UPLOAD as u8, u8::min(0x40, self.user_read_size)];

            // Send the extended command 0x27
            let res = self.send_request(crate::commands::Extended(0x27, &payload))?;

            // Ensure that the response is for the correct command
            if res.0 != 0x27 {
                return Err(crate::errors::DecodeError::ExpectedCommand(0x27, res.0));
            }

            // The response payload should be the data that we read, so copy it into the read buffer
            // Discarding the first byte like pros does
            self.read_buffer.extend(&res.1[1..]);

        }

        // The amount of data to read into the buf
        let data_len = usize::min(buf.len(), self.read_buffer.len());

        // Get the data from the read buffer
        let mut data = self.read_buffer[..data_len].to_vec();

        // Pad it to the length of buf with 0s
        data.resize(buf.len(), 0);

        // Strip the data from the read buffer
        self.read_buffer = self.read_buffer[data_len..].to_vec();

        // Copy the first bytes of the read_buffer into buf, maxing out at the length of buf.
        // We do this so no data is lost
        buf.copy_from_slice(&data);

        // Return the length of the data we read
        Ok(data_len)
    }

    /// Writes to the user serial port over the system port
    pub fn write_serial(&mut self, buf: &[u8]) -> Result<usize, crate::errors::DecodeError> {

        // Use a maximum default packet size of 224
        let max_size = 224;

        // Cut the data into bits that are at most max_size in length, and send each one one at a time
        let size = buf.len();

        for i in (0..size).step_by(max_size) {

            // Find ot how much data to send
            let packet_size = if i + max_size > size {
                size - i
            } else {
                max_size
            };

            // Put together the data into a custom extended command
            let mut payload = vec![0x01, 0xff];
            payload.extend_from_slice(&buf[i..i+packet_size]);

            // Send the command. This is the same as read, but we define the length of data to read as 0xff.
            let _ = self.send_request(crate::commands::Extended(0x27, &payload))?;

            // The result should be empty
        }

        // This code should send every byte
        Ok(size)
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

impl<S, U> std::io::Write for Device<S, U>
where S: Read + Write, U: Read + Write {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}