//! MQTT data types and representations. 
//! 
//! These types all map more or less ot rust data types directly,
//! and exist as a bridge to the binary-level protocol.
//! 
//! | MQTT type | rust type | Crate type | Description |
//! | --------- | --------- | ---------- | ----------- |
//! | Bits | [u8] | - | Big-Endian single byte |
//! | 2 Byte Int | [u16] | - | Unsigned 16-bit integer (Big-Endian) |
//! | 4 Byte Int | [u32] | - | Unsigned 32-bit integer (Big Endian) |
//! | Variable Byte Int | [u32] | [VariableByteInteger](self::integer::VariableByteInteger) | Unsigned big-endian integer represented from 8 to 24 bits, depending on the value |
//! | Binary Data | `Vec<u8>` or `&[u8]` | [BinaryData](self::bytes::BinaryData) | A sequence of bytes, max length is 65,535 |
//! | UTF-8 String | [String] | [UTF8String](self::string::UTF8String) |Max length 65,535 bytes (not characters!) |
//! | UTF-8 String pair | (String, String) | [UTF8StringPair](self::string::UTF8StringPair) | Length restrictions count per each individually |
//! 
//! Where "wrapper" structs exists for their respective rust data types, it is for necessary additional logic in 
//! encoding/decoding, such as the algorithm for [self::integer::VariableByteInteger] or additional length bytes for
//! Strings and binary data. 
//! 
//! # Integers
//! The simpler integer types (`u8`, `u16`, `u32`) will use whatever Endianness the platform is using, however they 
//! will always be Big-Endian in their encoded form.

mod bytes;
mod codes;
mod integer;
mod string;
mod qos;

pub use self::bytes::BinaryData;
pub use self::codes::ReasonCode;
pub use self::integer::VariableByteInteger;
pub use self::string::UTF8String;
pub use self::string::UTF8StringPair;
pub use self::qos::QoS;

/// A data type as defined in the MQTT spec.
/// 
/// TODO we'd really like to add bounds to to make sure implementations can be converted to and from binary, i.e.
/// `MqttDataType: Into<Vec<u8>> + TryFrom<&[u8]>`
pub trait MqttDataType {

    /// Returns the size in number of bytes that this type will use in a binary MQTT packet.
    fn encoded_len(&self) -> usize;
}
