use std::collections::HashMap;

use crate::{types::{ReasonCode, VariableByteInteger, MqttDataType, UTF8String, UTF8StringPair}, error::MqttError, packet::properties::{PropertyIdentifier, DataRepresentation}};

use super::{MqttControlPacket, PacketType, properties::{PropertyProcessor, MqttProperty}};

/// The first byte with packet identifier and flags is static for DISCONNECT packets
const FIRST_BYTE: u8 = 0b011100000;

/// A `DISCONNECT` message cleanly severs the connection between client and server.
/// 
/// May be sent by either the client or the server.
/// 
/// See [the spec](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901205)
#[derive(Debug, PartialEq)]
pub struct DisconnectPacket {
    
    pub reason_code: ReasonCode,

    pub properties: Option<DisconnectProperties>,
}

/// Optional properties in the `DISCONNECT` packet variable header.
#[derive(Debug, PartialEq)]
pub struct DisconnectProperties {

    pub session_expiry_interval: Option<u32>,

    /// 
    pub reason_string: Option<String>,

    /// Application-specific key-value elements.
    pub user_properties: HashMap<String, String>,

    ///
    pub server_reference: Option<String>,
}

impl Default for DisconnectPacket {
    /// Returns a [DisconnectPacket] with [reason code success](crate::types::ReasonCode) and no properties.
    fn default() -> Self {
        Self { reason_code: ReasonCode::Success, properties: None }
    }
}

impl TryFrom<&[u8]> for DisconnectPacket {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;
        if src[cursor] != FIRST_BYTE {
            return Err(MqttError::invalid_packet_identifier(DisconnectPacket::packet_type(), &src[0]))
        }

        cursor += 1;
        let remaining_length = VariableByteInteger::try_from(&src[cursor..])?;
        let remaining_length_value = remaining_length.encoded_len();

        // If the remaining length is 0, reason code success is assumed and there are no properties
        cursor += remaining_length_value;
        let (reason_code, properties)  = match remaining_length.value {
            0 => (ReasonCode::Success, None),
            _ => {
                let actual_length = src.len() - cursor;
                if remaining_length_value > actual_length {
                    return Err(MqttError::MalformedPacket(format!("Defined [{}] remaining length longer than actual [{}]", remaining_length_value, actual_length)))
                }

                let reason_code = ReasonCode::try_from(src[cursor])?;
                cursor += 1;
                let mut properties = DisconnectProperties::default();
                super::properties::parse_properties(&src[cursor..], &mut properties)?;
                (reason_code, Some(properties))
            }
        };
        
        Ok(DisconnectPacket { reason_code, properties})
    }
}

impl Into<Vec<u8>> for DisconnectPacket {
    fn into(self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();

        packet.push(FIRST_BYTE);
        
        packet.push(self.reason_code.into());

        let property_length = match self.properties {
            None => {
                println!("No properties, skipping");
                0
            },
            Some(props) => {
                let mut length = 0;
                if let Some(v) = props.session_expiry_interval {
                    length += super::properties::encode_and_append_property(
                        PropertyIdentifier::SessionExpiryInterval, 
                        DataRepresentation::FourByteInt(v), 
                        &mut packet);
                }
                if let Some(v) = props.reason_string {
                    length += super::properties::encode_and_append_property(
                        PropertyIdentifier::ReasonString, 
                        DataRepresentation::UTF8(UTF8String::from(v)), 
                        &mut packet);
                }
                if let Some(v) = props.server_reference {
                    length += super::properties::encode_and_append_property(
                        PropertyIdentifier::ServerReference, 
                        DataRepresentation::UTF8(UTF8String::from(v)), 
                        &mut packet);
                }
                for (k, v) in props.user_properties {
                    length += super::properties::encode_and_append_property(
                        PropertyIdentifier::UserProperty, 
                        DataRepresentation::UTF8Pair(UTF8StringPair::new(k, v)), 
                        &mut packet);
                }
                length
            }
        };

        // insert property length first
        super::encode_and_insert(VariableByteInteger::from(property_length), 2, &mut packet);

        // then the "remaining length"
        super::calculate_and_insert_length(&mut packet);

        packet
    }
}

impl MqttControlPacket<'_> for DisconnectPacket {
    fn packet_type() -> PacketType {
        PacketType::DISCONNECT
    }
}

impl Default for DisconnectProperties {
    fn default() -> Self {
        Self { 
            session_expiry_interval: None, 
            reason_string: None, 
            user_properties: HashMap::new(), 
            server_reference: None 
        }
    }
}

impl PropertyProcessor for DisconnectProperties {
    fn process(&mut self, property: MqttProperty) -> Result<(), MqttError> {
        match property.identifier {

            PropertyIdentifier::SessionExpiryInterval => {
                if let DataRepresentation::FourByteInt(v) = property.value {
                    self.session_expiry_interval = Some(v)
                }
            },
            PropertyIdentifier::ReasonString => {
                if let DataRepresentation::UTF8(v) = property.value {
                    self.reason_string = v.value
                }
            },
            PropertyIdentifier::UserProperty => {
                if let DataRepresentation::UTF8Pair(v) = property.value {
                    if let Some(s) = v.key.value {
                        self.user_properties.insert(s, v.value.value.unwrap_or(String::new()));
                    }
                }
            },
            PropertyIdentifier::ServerReference => {
                if let DataRepresentation::UTF8(s) = property.value {
                    self.server_reference = s.value
                }
            },
            _=> return Err(MqttError::ProtocolError(format!("Invalid property identifier [{:?}] for DISCONNECT", property.identifier)))
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::{vec, str::FromStr};

    use super::*;

    #[test]
    fn encode_and_decode() {
        let packet = DisconnectPacket::default();
        let encoded: Vec<u8> = packet.into();
        let decoded = DisconnectPacket::try_from(encoded.as_slice()).unwrap();
        assert_eq!(ReasonCode::Success, decoded.reason_code);
    }

    #[test]
    fn encode() {
        let disconnect = DisconnectPacket { reason_code: ReasonCode::NotAuthorized, properties: None };
        let binary: Vec<u8> = disconnect.into();
        let expected: Vec<u8> = vec![FIRST_BYTE, 2, 0x87, 0];
        assert_eq!(expected, binary);
    }

    #[test]
    fn encode_with_properties() {
        let mut properties = DisconnectProperties::default();
        properties.session_expiry_interval = Some(180);
        properties.reason_string = Some("because".into());
        let disconnect = DisconnectPacket { reason_code: ReasonCode::Success, properties: Some(properties) };

        let encoded: Vec<u8> = disconnect.into();
        let expected: Vec<u8> = vec![FIRST_BYTE, 17, 0, 15, 17, 0, 0, 0, 180, 31, 0, 7, 98, 101, 99, 97, 117, 115, 101];

        assert_eq!(expected, encoded);
    }

    #[test]
    fn decode() {
        let binary: Vec<u8> = vec![FIRST_BYTE, 5, 0, 0, 2, 3, 4]; // just adding a few dummy values after the reason code
        let disconnect = DisconnectPacket::try_from(&binary[..]).unwrap();
        assert_eq!(ReasonCode::Success, disconnect.reason_code);
    }

    #[test]
    fn decode_implicit_success() {
        let binary: Vec<u8> = vec![FIRST_BYTE, 0];
        let decoded = DisconnectPacket::try_from(binary.as_slice()).unwrap();
        assert_eq!(ReasonCode::Success, decoded.reason_code);
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
    fn decode_with_properties() {
        let binary: Vec<u8> = vec![FIRST_BYTE, 17, 0, 15, 17, 0, 0, 0, 180, 31, 0, 7, 98, 101, 99, 97, 117, 115, 101];
        let disconnect = DisconnectPacket::try_from(&binary[..]).expect("Unexpected error decoding DisconnectPacket");
        
        assert!(disconnect.properties.is_some());
        let properties = disconnect.properties.unwrap();
        assert_eq!(Some(180_u32), properties.session_expiry_interval);
        assert_eq!(Some(String::from_str("because").unwrap()), properties.reason_string);
    }

    #[test]
    fn wrong_packet_identifier() {
        let bin: Vec<u8> = vec![32, 1, 0];
        let res = DisconnectPacket::try_from(&bin[..]);
        assert!(res.is_err(), "expected a MalformedPacket error");
        assert_eq!(Some(MqttError::MalformedPacket(format!("Invalid packet identifier for DISCONNECT: 00100000"))), res.err());
        
    }
}