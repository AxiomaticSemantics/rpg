//! Network protocol

pub mod protocol;

use lightyear::shared::{config::SharedConfig, tick_manager::TickConfig};
use std::time::Duration;

// Use a port of 0 to automatically select a port
pub const PROTOCOL_ID: u64 = 0;
pub const SERVER_PORT: u16 = 4269;

pub const KEY: [u8; 32] = [0; 32];

pub fn shared_config() -> SharedConfig {
    SharedConfig {
        client_send_interval: Duration::default(),
        server_send_interval: Duration::from_millis(32),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 60.0),
        },
    }
}
