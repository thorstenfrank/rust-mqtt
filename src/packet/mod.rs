mod connect;
mod connack;

pub use self::connect::ConnectPacket;
pub use self::connack::ConnackPacket;

/// MQTT control packet types.
#[repr(u8)]
pub enum PacketType {
    CONNECT = 1,
    CONNACK = 2,
    /*PUBLISH = 3,
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
    AUTH = 15,*/
}

/// Common behavior for MQTT control packets.
pub trait MqttControlPacket {

    /// packet type
    fn packet_type() -> PacketType;
}

#[cfg(test)]
mod tests {
    use super::PacketType;

    #[test]
    fn test() {
        assert_eq!(1, PacketType::CONNECT as u8);
        assert_eq!(2, PacketType::CONNACK as u8);
    }
}