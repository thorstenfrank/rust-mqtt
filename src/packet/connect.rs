use crate::{VariableByteInteger, UTF8String};

use super::{MqttControlPacket, PacketType};

/// CONNECT
/// 
/// Fixed Header (packet type (1) | reserved (0)):
/// 0001 0000
/// [remaining length: VBI] (len(variable_header) + len(payload))
/// 
/// Variable Header:
///     Protocol Name ('MQTT') 
///     Protocol Level (5)
///     Connect Flags (username, pwd, will retain, will qos (2 bits), will flag, clean start, reserved)
///     Keep Alive (2 byte, KA interval in seconds)
///     Properties:
///         length: VBI
///         session expiry interval
///         receive maximum
///         max packet size
///         topic alias max
///         request response info
///         request problem info
///         user property*
///         auth method
///         auth data
/// 
/// Payload:
/// ClientID, Will Props, Will Topic, Will Payload, username, password
#[derive(Debug, PartialEq)]
pub struct ConnectPacket {
    client_id: UTF8String,
    protocol_level: u8,
    // will,
    // auth (username/password, auth method/data)
    // keep alive
    // properties
}

impl ConnectPacket {

    /// TODO add auto-generated client id
    pub fn new(client_id: UTF8String) -> Self {
        ConnectPacket { client_id, protocol_level: 5 }
    }

    fn remaining_length(&self) -> VariableByteInteger {
        let mut length: u32 = 11;
        // 11 is the min length, which includes:
        // protocol name (6)
        // protocol level (1)
        // connect flags (1)
        // keep alive (2)
        // properties (1)

        // clientID
        length += 2;
        length += self.client_id.len() as u32;

        VariableByteInteger { value: length }
    }
}

impl From<&[u8]> for ConnectPacket {
    fn from(src: &[u8]) -> Self {
        // FIXME determine actual position and length of the client id
        // validation, returning a Result, error handling, etc etc
        ConnectPacket::new(UTF8String::from(&src[13..]))
    }
}

impl Into<Vec<u8>> for ConnectPacket {

    fn into(self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        let mut length: Vec<u8> = self.remaining_length().into();

        // fixed header
        packet.push(0b00010000);
        packet.append(&mut length);

        // variable header
        //   - protocol name
        packet.append(&mut UTF8String::from_str("MQTT").into());
        
        //   - protocol version
        packet.push(self.protocol_level);

        // connect flags
        // FIXME
        packet.push(0b00000010);

        // keep alive
        // FIXME
        packet.push(0);
        packet.push(0);

        // properties
        // FIXME
        packet.push(0);

        // client id
        packet.append(&mut self.client_id.into());

        packet        
    }
}

impl MqttControlPacket for ConnectPacket {
    
    fn packet_type() -> PacketType {
        PacketType::CONNECT
    }

    fn payload_requirement() -> crate::types::YesNoMaybe {
        crate::types::YesNoMaybe::Required
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn encode() {
        let conn = ConnectPacket::new(UTF8String::from_str("WUPPDI"));

        let binary: Vec<u8> = conn.into();

        assert!(binary.len() > 0);

        let proto_name = "MQTT".as_bytes();
        let wuppdi = "WUPPDI".as_bytes();

        let expect: Vec<u8> = vec![
            0b00010000,
            19,
            0, 4, proto_name[0], proto_name[1], proto_name[2], proto_name[3],
            5,
            0b00000010,
            0, 0, // keep alive
            0,
            0, 6, wuppdi[0], wuppdi[1], wuppdi[2], wuppdi[3], wuppdi[4], wuppdi[5]
        ];

        assert_eq!(expect, binary);
    }

    #[test]
    fn decode() {
        let binary: Vec<u8> = vec![16,19,0,4,77,81,84,84,5,2,0,0,0,0,6,87,85,80,80,68,73];
        let decoded = ConnectPacket::from(&binary[..]);
        let expect = ConnectPacket::new(UTF8String::from_str("WUPPDI"));
        assert_eq!(expect, decoded);
    }
}