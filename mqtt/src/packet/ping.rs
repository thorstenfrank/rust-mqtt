use crate::error::MqttError;

use super::MqttControlPacket;

pub struct Pingreq {}

pub struct Pingresp {}

const PINGREQ: [u8; 2] = [0b11000000, 0];
const PINGRESP: [u8; 2] = [0b11010000, 0];

impl MqttControlPacket<'_> for Pingreq {
    fn packet_type() -> super::PacketType {
        super::PacketType::PINGREQ
    }
}

impl From<Pingreq> for Vec<u8> {
    fn from(_: Pingreq) -> Self {
        let mut result = Vec::with_capacity(2);
        result.push(PINGREQ[0]);
        result.push(PINGREQ[1]);
        result
    }
}

impl TryFrom<&[u8]> for Pingreq {
    type Error = MqttError;
    
    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        if src != &PINGREQ[..] {
            return Err(MqttError::MalformedPacket(format!("Invalid PINGREQ packet: {:?}", src)))
        }
        Ok(Pingreq{})
    }
}

impl MqttControlPacket<'_> for Pingresp {
    fn packet_type() -> super::PacketType {
        super::PacketType::PINGRESP
    }
}

impl From<Pingresp> for Vec<u8> {
    fn from(_: Pingresp) -> Self {
        let mut result = Vec::with_capacity(2);
        result.push(0b11010000);
        result.push(0);
        result
    }
}

impl TryFrom<&[u8]> for Pingresp {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        if src != &PINGRESP[..] {
            return Err(MqttError::MalformedPacket(format!("Invalid PINGRESP packet: {:?}", src)))
        }
        Ok(Pingresp{})
    }
}