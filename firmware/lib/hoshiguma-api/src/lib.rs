#![cfg_attr(not(feature = "std"), no_std)]

mod api;
pub use api::*;

mod endpoints;
pub use endpoints::*;

mod types;
pub use types::*;

#[cfg(feature = "device-cooler")]
pub mod cooler;

#[cfg(feature = "device-hmi")]
pub mod hmi;

#[cfg(feature = "device-rear-sensor-board")]
pub mod rear_sensor_board;

#[cfg(feature = "device-telemetry-bridge")]
pub mod telemetry_bridge;
