#![no_std]

#[cfg(any(feature = "client", feature = "host"))]
pub mod comms;
