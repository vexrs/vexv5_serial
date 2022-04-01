
use std::io::{Read, Write};


/// Wraps an object with Read + Write traits implemented
/// to provide an implementation of the VEX V5 Protocol.
pub struct V5Protocol<T>
    where T: Read + Write {
    /// The read/write object to wrap
    /// This can be a file, serial port, socket, or anything else.
    /// This struct does not care.
    wraps: T,
}

impl<T> V5Protocol<T>
    where T: Read + Write {
    
    /// Creates a new V5Protocol object
    pub fn new(wraps: T) -> Self {
        V5Protocol {
            wraps,
        }
    }

    
}