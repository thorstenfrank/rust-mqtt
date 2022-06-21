use std::fmt::{self, Display};

#[derive(Debug)]
pub enum MqttError {
    Message(String),
}

impl std::error::Error for MqttError {}

impl Display for MqttError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MqttError::Message(msg) => formatter.write_str(msg),
            //_ => formatter.write_str("general error"),
        }
    }
}