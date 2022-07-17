use crate::{types::ReasonCode, error::MqttError};

use super::{MqttControlPacket, PacketType};

/// The first byte with packet identifier and flags is static for DISCONNECT packets
const FIRST_BYTE: u8 = 0b011100000;

/// DISCONNECT
/// 
/// Fixed header (packet type 14 | reserved 0)
/// 1110 0000
/// remaining length
/// 
/// Variable Header
///     reason code (1 byte)
///     properties
/// 
/// NO payload
#[derive(Debug)]
pub struct DisconnectPacket {
    pub reason_code: ReasonCode,
    // properties...
}

impl TryFrom<&[u8]> for DisconnectPacket {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        if src[0] != FIRST_BYTE {
            return Err(MqttError::invalid_packet_identifier(DisconnectPacket::packet_type(), &src[0]))
        }
        let reason_code = ReasonCode::try_from(src[2])?;
        Ok(DisconnectPacket { reason_code})
    }
}

impl Into<Vec<u8>> for DisconnectPacket {
    fn into(self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();

        packet.push(FIRST_BYTE);
        
        packet.push(self.reason_code.into());
        packet.push(0); // properties length


        super::calculate_and_insert_length(&mut packet);

        packet
    }
}

impl MqttControlPacket for DisconnectPacket {
    fn packet_type() -> PacketType {
        PacketType::DISCONNECT
    }
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    #[test]
    fn encode() {
        let disconnect = DisconnectPacket { reason_code: ReasonCode::Success };
        let binary: Vec<u8> = disconnect.into();
        let expected: Vec<u8> = vec![224, 2, 0, 0];
        assert_eq!(expected, binary);
    }

    #[test]
    fn decode() {
        let binary: Vec<u8> = vec![224, 5, 0, 1, 2, 3, 4]; // just adding a few dummy values after the reason code
        let disconnect = DisconnectPacket::try_from(&binary[..]).unwrap();
        assert_eq!(ReasonCode::Success, disconnect.reason_code);
    }

    #[test]
    fn decode_reason_code_unspecified_error() {
        let binary: Vec<u8> = vec![224, 2, 128, 0];
        let disconnect = DisconnectPacket::try_from(&binary[..]).unwrap();
        assert_eq!(ReasonCode::UnspecifiedError, disconnect.reason_code);
    }

    #[test]
    fn decode_reason_code_with_will() {
        let binary: Vec<u8> = vec![224, 2, 4, 0];

        // FIXME: Reason Code 4 (disconnect with will message) is not yet implemented, which is why we're
        // expecting an undefined error for now
        let disconnect = DisconnectPacket::try_from(&binary[..]);
        assert!(disconnect.is_err());
    }

    #[test]
    fn wrong_packet_identifier() {
        let bin: Vec<u8> = vec![32, 1, 0];
        let res = DisconnectPacket::try_from(&bin[..]);
        assert!(res.is_err(), "expected a MalformedPacket error");
        assert_eq!(Some(MqttError::MalformedPacket(format!("Invalid packet identifier for DISCONNECT: 00100000"))), res.err());
        
    }
}