mod connect;
mod connack;
mod disconnect;

use std::fmt::Display;

use crate::error::MqttError;
use crate::types::YesNoMaybe;

pub use self::connect::ConnectPacket;
pub use self::connack::ConnackPacket;

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
pub trait MqttControlPacket {

    /// packet type
    fn packet_type() -> PacketType;

    fn payload_requirement() -> YesNoMaybe;
}

#[cfg(test)]
mod tests {
    use crate::error::MqttError;

    use super::PacketType;

    #[test]
    fn test() {
        assert_eq!(1, PacketType::CONNECT as u8);
        assert_eq!(2, PacketType::CONNACK as u8);
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