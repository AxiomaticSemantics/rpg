//! Network protocol

pub mod protocol;

// Use a port of 0 to automatically select a port
pub const PROTOCOL_ID: u64 = 0;
pub const SERVER_PORT: u16 = 4269;

pub const KEY: [u8; 32] = [0; 32];
