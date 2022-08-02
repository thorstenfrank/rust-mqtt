//! Representations of MQTT data packets.
//! Includes serialization and deserialization of packets into and from binary.
//! 
//! The "API" of these packets all use standard rust data types that map directly to MQTT types. See the 
//! [`type` module-level documentation](crate::types). These protocol-specific types are used for encoding/decoding 
//! only..
//! 
//! The primary structs representing the [PacketType]s group fields together in sub-structs as they seem most logical
//! to use from an API perspective. There is no separation into fixed header, variable header and payload in the way 
//! the spec makes these distinctions. It is the internal encoding/decoding logic's responsibility to make sure these
//! are mapped properly.
//! 
//! # QoS and Message Flow
//! 
//! ## QoS 0 (At most once)
//! `DUP` must also be 0.
//! 
//! - Sender: `PUBLISH`
//! - Receiver: no response
//! 
//! ## QoS 1 (At least once)
//! - Sender: `PUBLISH`
//! - Receiver: `PUBACK`
//! 
//! ## QoS 2 (Exactly once)
//! - Sender: `PUBLISH`
//! - Receiver: `PUBREC`
//! - Sender: `PUBREL`
//! - Reciever: `PUBCOMP`

mod connect;
mod connack;
mod disconnect;
mod properties;
mod puback;
mod pubcomp;
mod publish;
mod pubrec;
mod pubrel;
mod subscribe;

use std::fmt::Display;

use crate::error::MqttError;
use crate::types::{VariableByteInteger, MqttDataType};

pub use self::connect::{Connect, ConnectProperties, LastWill, WillProperties};
pub use self::connack::{Connack, ConnackProperties};
pub use self::disconnect::{Disconnect, DisconnectProperties};
pub use self::puback::{Puback, PubackProperties};
pub use self::pubcomp::{Pubcomp, PubcompProperties};
pub use self::publish::{Publish, PublishProperties};
pub use self::pubrec::{Pubrec, PubrecProperties};
pub use self::pubrel::{Pubrel, PubrelProperties};
pub use self::subscribe::{Subscribe, SubscribeProperties};

/// MQTT control packet types.
#[derive(Debug, PartialEq, Eq)]
pub enum PacketType {
    CONNECT = 1,
    CONNACK = 2,
    PUBLISH = 3,
    PUBACK = 4,
    PUBREC = 5,
    PUBREL = 6,
    PUBCOMP = 7,
    SUBSCRIBE = 8,
    SUBACK = 9,
    UNSUBSCRIBE = 10,
    UNSUBACK = 11,
    PINGREQ = 12,
    PINGRESP = 13,
    DISCONNECT = 14,
    AUTH = 15,
}

impl TryFrom<u8> for PacketType {
    type Error = MqttError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let shifted = value >> 4;

        match shifted {
            1 => Ok(PacketType::CONNECT),
            2 => Ok(PacketType::CONNACK),
            3 => Ok(PacketType::PUBLISH),
            4 => Ok(PacketType::PUBACK),
            5 => Ok(PacketType::PUBREC),
            6 => Ok(PacketType::PUBREL),
            7 => Ok(PacketType::PUBCOMP),
            8 => Ok(PacketType::SUBSCRIBE),
            9 => Ok(PacketType::SUBACK),
            10 => Ok(PacketType::UNSUBSCRIBE),
            11 => Ok(PacketType::UNSUBACK),
            12 => Ok(PacketType::PINGREQ),
            13 => Ok(PacketType::PINGRESP),
            14 => Ok(PacketType::DISCONNECT),
            15 => Ok(PacketType::AUTH),
            _=> Err(MqttError::Message(format!("undefined packet type: {}", shifted))),
        }
    }
}

impl Display for PacketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            PacketType::CONNECT => write!(f, "CONNECT"),
            PacketType::CONNACK => write!(f, "CONNACK"),
            PacketType::PUBLISH => write!(f, "PUBLISH"),
            PacketType::PUBACK => write!(f, "PUBACK"),
            PacketType::PUBREC => write!(f, "PUBREC"),
            PacketType::PUBREL => write!(f, "PUBREL"),
            PacketType::PUBCOMP => write!(f, "PUBCOMP"),
            PacketType::SUBSCRIBE => write!(f, "SUBSCRIBE"),
            PacketType::SUBACK => write!(f, "SUBACK"),
            PacketType::UNSUBSCRIBE => write!(f, "UNSUBSCRIBE"),
            PacketType::UNSUBACK => write!(f, "UNSUBACK"),
            PacketType::PINGREQ => write!(f, "PINGREQ"),
            PacketType::PINGRESP => write!(f, "PINGRESP"),
            PacketType::DISCONNECT => write!(f, "DISCONNECT"),
            PacketType::AUTH => write!(f, "AUTH"),
        }
    }
}

/// Common behavior for MQTT control packets.
/// 
/// At the very least, it is expected that a packet can be transformed into and parsed from binary format.
pub trait MqttControlPacket<'a>: Into<Vec<u8>> + TryFrom<&'a [u8]> {
    
    /// Not sure we really need this...
    fn packet_type() -> PacketType;

}

/// Contains an optional decoding result along with the number of bytes "used" during decoding, even if the result
/// is `None`.
pub struct DecodingResult<T> {
    bytes_read: usize,
    value: Option<T>,
}

impl <T>DecodingResult<T> {

    pub fn bytes_read(&self) -> usize {
        self.bytes_read
    }

    pub fn value(self) -> Option<T> {
        self.value
    }
}

/// Essentially an extended version of `TryFrom<&[u8]>` that allows returning an "empty" result while still giving 
/// the caller information about how many bytes were actually used during decoding.
/// This is relevant especially for decoding elements that always have some footprint in the binary packet, such as
/// properties where the very least is a single byte representing length `0`.
pub trait Decodeable: Sized {

    /// Does what it says. May or may not return an actual value.
    fn decode(src: &[u8]) -> Result<DecodingResult<Self>, crate::error::MqttError>;
    
}

/// Decodes a [VariableByteInteger](crate::types::VariableByteInteger) from the beginning of the slice and compares
/// the decoded value against the actual remaining length of the slice. If the remaining slice is shorter than the
/// specified one, an error is returned.
fn remaining_length(src: &[u8]) -> Result<VariableByteInteger, MqttError> {
    let remain_len = VariableByteInteger::try_from(&src[..])?;
    let actual_len = (src.len() - remain_len.encoded_len()) as u32;

    if remain_len.value > actual_len {
        return Err(MqttError::MalformedPacket(
            format!("Message too short, expected {}, but was {} bytes", remain_len.value, actual_len)))
    }

    Ok(remain_len)
}

/// Converts `val` into two Big-Endian bytes and appends them to `vec`.
/// TODO this should be moved somewhere together with all the other general read, parse, push and encode functions
fn push_be_u16(val: u16, vec: &mut Vec<u8>) {
    for b in val.to_be_bytes() {
        vec.push(b)
    }
}

/// Converts the first two bytes of the slice into a big-endian u16.
/// returns an error if the slice is shorter than 2 bytes
fn u16_from_be_bytes(src: &[u8]) -> Result<u16, MqttError> {
    let index = std::mem::size_of::<u16>();
    if index >= src.len() {
        return Err(MqttError::Message(format!("Source slice too short for u16!")))
    }

    let (int_bytes, _) = src.split_at(index);
    match int_bytes.try_into() {
        Ok(a) => Ok(u16::from_be_bytes(a)),
        Err(e) => Err(MqttError::Message(format!("Error decoding u16: {:?}", e))),
    }
}

/// Converts the first four bytes of the slice into a big-endian u32.
/// returns an error if the slice is shorter than 4 bytes
fn u32_from_be_bytes(src: &[u8]) -> Result<u32, MqttError> {
    let index = std::mem::size_of::<u32>();
    if index >= src.len() {
        return Err(MqttError::Message(format!("Source slice too short for u16!")))
    }

    let (int_bytes, _) = src.split_at(index);
    match int_bytes.try_into() {
        Ok(a) => Ok(u32::from_be_bytes(a)),
        Err(e) => Err(MqttError::Message(format!("Error decoding u32: {:?}", e))),
    }
}

/// Converts `val` into four Big-Endian bytes and appends them to `vec`.
/// TODO this should be moved somewhere together with all the other general read, parse, push and encode functions
fn push_be_u32(val: u32, vec: &mut Vec<u8>) {
    for b in val.to_be_bytes() {
        vec.push(b)
    }
}

/// The fixed header part of an MQTT packet includes the 'remaining length' starting with the second byte
const LENGTH_START_INDEX: usize = 1;

/// Subtracts 1 from the vec's length (because we're assuming the first byte is the packet type and flags), creates a
/// [`VariableByteInteger`] from it and then calls [`insert()`].
fn calculate_and_insert_length(packet: &mut Vec<u8>) {
    encode_and_insert(VariableByteInteger { value: (packet.len() - 1) as u32 }, LENGTH_START_INDEX, packet)
}

/// Encodes `val` into its binary representation and appends the resulting bytes to `vec`.
fn encode_and_append<T: Into<Vec<u8>>>(val: T, vec: &mut Vec<u8>) {
    vec.append(&mut val.into())
}

/// Encodes `val` into its binary representation and then inserts those bytes at the specified index.
/// To encode and append at the end of the 
fn encode_and_insert<T: Into<Vec<u8>>>(val: T, start_index: usize, vec: &mut Vec<u8>) {
    let mut index = start_index;
    let encoded: Vec<u8> = val.into();
    for b in encoded {
        vec.insert(index, b);
        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::error::MqttError;

    use super::{PacketType, calculate_and_insert_length};

    #[test]
    fn calculate_and_insert() {
        let mut short: Vec<u8> = vec![0; 45];
        calculate_and_insert_length(&mut short);

        assert_eq!(46, short.len());
        assert_eq!(44, short[1]);

        let mut long: Vec<u8> = vec![0; 2_097_151];
        calculate_and_insert_length(&mut long);

        assert_eq!(2_097_154, long.len());
        assert_eq!(254, long[1]);
        assert_eq!(255, long[2]);
        assert_eq!(127, long[3]);
        assert_eq!(0, long[4]);
        //do_test_encode_vbi(2097151, vec![0xFF, 0xFF, 0x7F]);
    }

    #[test]
    fn test_packet_from_u8() {
        assert_eq!(Some(MqttError::Message("undefined packet type: 0".to_string())), PacketType::try_from(0b00000000).err());

        do_test_packet_from_u8(0b00010000, PacketType::CONNECT);
        // just doing this to test that the last four bits are ignored
        do_test_packet_from_u8(0b00011111, PacketType::CONNECT);
        do_test_packet_from_u8(0b00100000, PacketType::CONNACK);
        do_test_packet_from_u8(0b00110000, PacketType::PUBLISH);
        do_test_packet_from_u8(0b01000000, PacketType::PUBACK);
        do_test_packet_from_u8(0b01010000, PacketType::PUBREC);
        do_test_packet_from_u8(0b01100000, PacketType::PUBREL);
        do_test_packet_from_u8(0b01110000, PacketType::PUBCOMP);
        do_test_packet_from_u8(0b10000000, PacketType::SUBSCRIBE);
        do_test_packet_from_u8(0b10010000, PacketType::SUBACK);
        do_test_packet_from_u8(0b10100000, PacketType::UNSUBSCRIBE);
        do_test_packet_from_u8(0b10110000, PacketType::UNSUBACK);
        do_test_packet_from_u8(0b11000000, PacketType::PINGREQ);
        do_test_packet_from_u8(0b11010000, PacketType::PINGRESP);
        do_test_packet_from_u8(0b11100000, PacketType::DISCONNECT);
        do_test_packet_from_u8(0b11110000, PacketType::AUTH);
        do_test_packet_from_u8(0b11110101, PacketType::AUTH);
    }

    fn do_test_packet_from_u8(numeric: u8, expected: PacketType) {
        let res = PacketType::try_from(numeric);
        assert_eq!(expected, res.unwrap());
    }

}