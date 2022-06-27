use std::fmt::{self, Display};

/// 
#[derive(Debug, Clone, PartialEq)]
pub enum MqttError {
    
    /// 
    MalformedPacket(String),

    /// a general-use error string
    Message(String),
}

impl std::error::Error for MqttError {}

impl Display for MqttError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MqttError::MalformedPacket(detail) => formatter.write_fmt(format_args!("Malformed Packet: {}", detail)),
            MqttError::Message(msg) => formatter.write_str(msg),
            //_ => formatter.write_str("general error"),
        }
    }
}