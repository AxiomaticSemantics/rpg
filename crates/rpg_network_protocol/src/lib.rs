//! Network protocol

pub mod protocol;

use lightyear::shared::{config::SharedConfig, tick_manager::TickConfig};
use std::time::Duration;

// Use a port of 0 to automatically select a port
pub const CLIENT_PORT: u16 = 0;
pub const SERVER_PORT: u16 = 5000;
pub const PROTOCOL_ID: u64 = 0;

pub const KEY: [u8; 32] = [0; 32];

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Transports {
    Udp,
}

pub fn shared_config() -> SharedConfig {
    SharedConfig {
        enable_replication: false,
        client_send_interval: Duration::default(),
        server_send_interval: Duration::from_millis(32),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 60.0),
        },
    }
}
