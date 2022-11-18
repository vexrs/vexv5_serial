// Module that contains all commands that can be sent to the v5

mod kv;
pub use kv::{KVRead, KVWrite};

mod extended;
pub use extended::{Extended, ExtendedResponse};

mod simple;
pub use simple::{Simple, SimpleResponse};


/// A command trait that every command implements
pub trait Command {
    type Response;
    /// Encodes the library->v5 request
    /// 
    /// Implementation is specific to each command, but generally it returnes the data in the command's structure
    /// parsed into a Vec<u8>
    fn encode_request(self) -> Vec<u8>;

    /// Decodes a response from a stream
    /// 
    /// # Arguments
    /// 
    /// * `stream` - The stream implementing the `Read` trait that the decoder will read from
    /// * `timeout` - Maximum amount of time that the reader will wait before it recieves a packet header.
    fn decode_stream<T: std::io::Read>(stream: &mut T, timeout: std::time::Duration) -> Result<Self::Response, crate::errors::DecodeError>;
}