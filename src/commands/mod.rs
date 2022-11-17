// Module that contains all commands that can be sent to the v5

mod kv;
pub use kv::{KVRead, KVReadResponse};

mod extended;
pub use extended::{Extended, ExtendedResponse};

mod simple;
pub use simple::{Simple, SimpleResponse};

/// A command trait that every command implements
pub trait Command {
    type Response;
    /// Encodes the library->v5 request
    fn encode_request(self) -> Vec<u8>;

    /// Decodes the payload of a v5->library response
    fn decode_response_payload(payload: Vec<u8>) -> Result<Self::Response, crate::errors::DecodeError>;
}