// Module that contains all commands that can be sent to the v5

mod kv;
pub use kv::{KVRead, KVReadResponse, KVWrite, KVWriteResponse};

mod extended;
pub use extended::{Extended, ExtendedResponse};

mod simple;
pub use simple::{Simple, SimpleResponse};

/// A command trait that every command implements
pub trait Command {
    type Response;
    /// Encodes the library->v5 request
    fn encode_request(self) -> Vec<u8>;

    /// Decodes a response from a stream
    fn decode_stream<T: std::io::Read>(stream: &mut T, timeout: std::time::Duration) -> Result<Self::Response, crate::errors::DecodeError>;
}