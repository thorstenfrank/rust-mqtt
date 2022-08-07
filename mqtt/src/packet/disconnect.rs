use std::collections::HashMap;

use mqtt_derive::MqttProperties;
use crate::{types::{ReasonCode, MqttDataType}, error::MqttError};

use super::{MqttControlPacket, PacketType, Decodeable, DecodingResult, remaining_length};

/// The first byte with packet identifier and flags is static for DISCONNECT packets
const FIRST_BYTE: u8 = 0b11100000;

/// A `DISCONNECT` message cleanly severs the connection between client and server.
/// 
/// May be sent by either the client or the server.
/// 
/// See [the spec](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901205)
#[derive(Debug, PartialEq)]
pub struct Disconnect {
    
    /// Details about the disconnect.
    pub reason_code: ReasonCode,

    /// MQTT5 optional properties.
    pub properties: Option<DisconnectProperties>,
}

/// Optional properties in the `DISCONNECT` packet variable header.
#[derive(Debug, PartialEq, MqttProperties)]
pub struct DisconnectProperties {

    /// Sets the expiration for the current session for a potential re-connect.
    /// Only relevant for client-side disconnects!
    pub session_expiry_interval: Option<u32>,

    /// A human-readable explanation of the disconnect, if applicable.
    pub reason_string: Option<String>,

    /// Application-specific key-value elements.
    pub user_property: HashMap<String, String>,

    /// This is usually only populated if the `reason code` is `Server Busy` to indicate a potential other server
    /// to try out.
    pub server_reference: Option<String>,
}

impl Default for Disconnect {
    /// Returns a [`Disconnect`] with [reason code success](crate::types::ReasonCode) and no properties.
    fn default() -> Self {
        Self { reason_code: ReasonCode::Success, properties: None }
    }
}

impl TryFrom<&[u8]> for Disconnect {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;
        if src[cursor] != FIRST_BYTE {
            return Err(MqttError::invalid_packet_identifier(Disconnect::packet_type(), &src[0]))
        }

        cursor += 1;
        let remaining_length = remaining_length(&src[cursor..])?;
        let remaining_length_value = remaining_length.encoded_len();

        // If the remaining length is 0, reason code success is assumed and there are no properties
        cursor += remaining_length_value;
        let (reason_code, properties)  = match remaining_length.value {
            0 => (ReasonCode::Success, None),
            _ => {
                let reason_code = ReasonCode::try_from(src[cursor])?;
                cursor += 1;

                let prop_res: DecodingResult<DisconnectProperties> = DisconnectProperties::decode(&src[cursor..])?;

                (reason_code, prop_res.value())
            }
        };
        
        Ok(Disconnect { reason_code, properties})
    }
}

impl From<Disconnect> for Vec<u8> {
    fn from(src: Disconnect) -> Self {
        let mut packet: Vec<u8> = Vec::new();

        packet.push(FIRST_BYTE);
        
        packet.push(src.reason_code.into());

        if let Some(props) = src.properties {
            packet.append(&mut props.into());
        } else {
            packet.push(0); // no properties => just add a zero
        }

        // then the "remaining length"
        super::calculate_and_insert_length(&mut packet);

        packet
    }
}

impl MqttControlPacket<'_> for Disconnect {
    fn packet_type() -> PacketType {
        PacketType::DISCONNECT
    }
}

#[cfg(test)]
mod tests {

    use std::{vec, str::FromStr};

    use super::*;

    #[test]
    fn encode_and_decode() {
        let packet = Disconnect::default();
        let encoded: Vec<u8> = packet.into();
        let decoded = Disconnect::try_from(encoded.as_slice()).unwrap();
        assert_eq!(ReasonCode::Success, decoded.reason_code);
    }

    #[test]
    fn encode() {
        let disconnect = Disconnect { reason_code: ReasonCode::NotAuthorized, properties: None };
        let binary: Vec<u8> = disconnect.into();
        let expected: Vec<u8> = vec![FIRST_BYTE, 2, 0x87, 0];
        assert_eq!(expected, binary);
    }

    #[test]
    fn encode_with_properties() {
        let mut properties = DisconnectProperties::default();
        properties.session_expiry_interval = Some(180);
        properties.reason_string = Some("because".into());
        let disconnect = Disconnect { reason_code: ReasonCode::Success, properties: Some(properties) };

        let encoded: Vec<u8> = disconnect.into();
        let expected: Vec<u8> = vec![FIRST_BYTE, 17, 0, 15, 17, 0, 0, 0, 180, 31, 0, 7, 98, 101, 99, 97, 117, 115, 101];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn decode() {
        let binary: Vec<u8> = vec![FIRST_BYTE, 5, 0, 0, 2, 3, 4]; // just adding a few dummy values after the reason code
        let disconnect = Disconnect::try_from(&binary[..]).unwrap();
        assert_eq!(ReasonCode::Success, disconnect.reason_code);
    }

    #[test]
    fn decode_implicit_success() {
        let binary: Vec<u8> = vec![FIRST_BYTE, 0];
        let decoded = Disconnect::try_from(binary.as_slice()).unwrap();
        assert_eq!(ReasonCode::Success, decoded.reason_code);
    }

    #[test]
    fn decode_reason_code() {
        let binary: Vec<u8> = vec![224, 1, 130];
        let disconnect = Disconnect::try_from(&binary[..]).unwrap();
        assert_eq!(ReasonCode::ProtocolError, disconnect.reason_code);        
    }

    #[test]
    fn decode_reason_code_unspecified_error() {
        let binary: Vec<u8> = vec![224, 2, 128, 0];
        let disconnect = Disconnect::try_from(&binary[..]).unwrap();
        assert_eq!(ReasonCode::UnspecifiedError, disconnect.reason_code);
    }

    #[test]
    fn decode_reason_code_with_will() {
        let binary: Vec<u8> = vec![224, 2, 4, 0];
        let disconnect = Disconnect::try_from(&binary[..]).unwrap();
        assert_eq!(ReasonCode::DisconnectWithWill, disconnect.reason_code);
    }

    #[test]
    fn decode_with_properties() {
        let binary: Vec<u8> = vec![FIRST_BYTE, 17, 0, 15, 17, 0, 0, 0, 180, 31, 0, 7, 98, 101, 99, 97, 117, 115, 101];
        let disconnect = Disconnect::try_from(&binary[..]).expect("Unexpected error decoding DisconnectPacket");
        
        assert!(disconnect.properties.is_some());
        let properties = disconnect.properties.unwrap();
        assert_eq!(Some(180_u32), properties.session_expiry_interval);
        assert_eq!(Some(String::from_str("because").unwrap()), properties.reason_string);
    }

    #[test]
    fn wrong_packet_identifier() {
        let bin: Vec<u8> = vec![32, 1, 0];
        let res = Disconnect::try_from(&bin[..]);
        assert!(res.is_err(), "expected a MalformedPacket error");
        assert_eq!(Some(MqttError::MalformedPacket(format!("Invalid packet identifier for DISCONNECT: 00100000"))), res.err());
        
    }

    #[test]
    fn disconnect_properties_default() {
        let packet = DisconnectProperties::default();

        assert!(packet.reason_string.is_none());
        assert!(packet.server_reference.is_none());
        assert!(packet.session_expiry_interval.is_none());
        assert!(packet.user_property.is_empty());
    }

    #[test]
    fn encode_properties() {
        let mut props = DisconnectProperties::default();
        props.user_property.insert("wuppdi".to_string(), "heppes".to_string());
        props.session_expiry_interval = Some(120);
        props.reason_string = Some(String::from("Because you are a test"));

        let vec: Vec<u8> = props.into();
        assert!(!vec.is_empty());
        assert_eq!(48, vec.len());
        
    }
}