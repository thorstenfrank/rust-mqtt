//! Custom error types used throughout the crate.

use std::fmt::{self, Display};

use crate::packet::PacketType;

/// Custom error types.
/// 
/// TODO: both malformed packet and protocol error should contain `reason codes`!
#[derive(Debug, Clone, PartialEq)]
pub enum MqttError {
    
    /// Syntactical error indicating that a control packet could not be fully parsed.
    /// See MQTT spec `1.2` and `4.13`.
    MalformedPacket(String),

    /// Used for packets containing invalid or inconsistent data.
    /// See MQTT spec `1.2` and `4.13`.
    ProtocolError(String),

    /// A general-use error in cases where none of the more specific ones fit.
    Message(String),
}

impl MqttError {
    pub fn invalid_packet_identifier(packet_type: PacketType, first_byte: &u8) -> Self {
        MqttError::MalformedPacket(format!("Invalid packet identifier for {}: {:08b}", packet_type, first_byte))
    }
}

impl std::error::Error for MqttError {}

impl Display for MqttError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MqttError::MalformedPacket(detail) => formatter.write_fmt(format_args!("Malformed Packet: {}", detail)),
            MqttError::ProtocolError(detail) => formatter.write_fmt(format_args!("Protocol Error: {}", detail)),
            MqttError::Message(msg) => formatter.write_str(msg),
            //_ => formatter.write_str("general error"),
        }
    }
}