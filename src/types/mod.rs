mod bytes;
mod codes;
mod integer;
mod string;
mod qos;

pub use self::bytes::BinaryData;
pub use self::codes::ReasonCode;
pub use self::integer::{VariableByteInteger, push_be_u16, push_be_u32};
pub use self::string::UTF8String;
pub use self::qos::QoS;

/// A data type as defined in the MQTT spec.
pub trait MqttDataType {

    /// Returns the size in number of bytes that this type will use in a binary MQTT packet.
    fn encoded_len(&self) -> usize;
}