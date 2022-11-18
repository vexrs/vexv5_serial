//! Contains metadata about the V5

/// Enum that represents the channel
/// for the V5 Controller
pub enum V5ControllerChannel {
    /// Used when wirelessly controlling the 
    /// V5 Brain
    PIT = 0x00,
    /// Used when wirelessly uploading data to the V5
    /// Brain
    UPLOAD = 0x01,
}
