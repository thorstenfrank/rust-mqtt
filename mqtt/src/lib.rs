//! A library representing the MQTT protocol with a focus on encoding to and decoding from bytes.
//! 
//! Whenever documentation in this crate refers to "the specification", it refers to the official 
//! [OASIS MQTTv5 standard](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).
//! 
//! This crate does not use external dependencies, and does not support `no_std`.

pub mod error;
pub mod packet;
pub mod types;