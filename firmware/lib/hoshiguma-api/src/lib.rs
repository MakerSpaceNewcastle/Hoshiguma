#![cfg_attr(not(feature = "std"), no_std)]

mod api;
pub use api::*;

mod types;
pub use types::*;

#[cfg(feature = "device-cooler")]
pub mod cooler;

#[cfg(feature = "device-hmi")]
pub mod hmi;

#[cfg(feature = "device-rear-sensor-board")]
pub mod rear_sensor_board;

#[cfg(feature = "device-telemetry-module")]
pub mod telemetry_module;
