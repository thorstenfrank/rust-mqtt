use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{types::{ReasonCode, MqttDataType}, error::MqttError, packet::Decodeable};

use super::MqttControlPacket;

/// `PUBREL` is the response to a [`PUBREC`](crate::packet::Pubrec). 
/// Applies only to messages published with [crate::types::QoS::ExactlyOnce].
/// 
/// The sequence of messages for QoS 2 is as follows:
/// - `PUBLISH` -->
/// - `PUBREC` <--
/// - `PUBREL` -->
/// - `PUBCOMP` <-- 
#[derive(Debug)]
pub struct Pubrel {
    pub packet_identifier: u16,
    pub reason_code: ReasonCode,
    pub properties: Option<PubrelProperties>,
}

#[derive(Debug, MqttProperties)]
pub struct PubrelProperties {
    pub reason_string: Option<String>,
    pub user_property: HashMap<String, String>,
}

impl MqttControlPacket<'_> for Pubrel {
    fn packet_type() -> super::PacketType {
        super::PacketType::PUBREL
    }
}

/// Fixed first byte of the header
const FIRST_BYTE: u8 = 0b01100010;

impl Pubrel {

    pub fn new(packet_identifier: u16, reason_code: ReasonCode) -> Result<Self, MqttError> {
        Self::validate_reason_code(&reason_code)?;
        Ok(Self { packet_identifier, reason_code, properties: None })
    }

    fn validate_reason_code(reason_code: &ReasonCode) -> Result<(), MqttError> {
        match reason_code {
            ReasonCode::Success | 
            ReasonCode::PacketIdentifierNotFound => Ok(()),
            els => Err(MqttError::ProtocolError(format!("Invalid reason code [{}] for PUBREL", u8::from(*els)))),
        }
    }
}

impl From<Pubrel> for Vec<u8> {
    fn from(pubrel: Pubrel) -> Self {
        let mut result: Vec<u8> = Vec::new();

        result.push(FIRST_BYTE);
        super::push_be_u16(pubrel.packet_identifier, &mut result);
        result.push(pubrel.reason_code.into());

        match pubrel.properties {
            Some(props) => result.append(&mut props.into()),
            None => result.push(0),
        }

        super::calculate_and_insert_length(&mut result);

        result
    }
}

impl TryFrom<&[u8]> for Pubrel {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;

        match src[cursor] {
            FIRST_BYTE => cursor += 1,
            els => return Err(MqttError::MalformedPacket(format!("First byte is not a PUBREL one: {:b}", els)))
        }

        let remain_len = super::remaining_length(&src[cursor..])?;
        cursor += remain_len.encoded_len();

        let packet_identifier = super::u16_from_be_bytes(&src[cursor..])?;
        cursor += packet_identifier.encoded_len();

        let reason_code = ReasonCode::try_from(src[cursor])?;
        cursor += reason_code.encoded_len();
        
        let mut result = Self::new(packet_identifier, reason_code)?;
        result.properties = PubrelProperties::decode(&src[cursor..])?.value();
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn encode_and_decode() {
        let pubrel = Pubrel::new(123, ReasonCode::Success).unwrap();
        let encoded: Vec<u8> = pubrel.into();

        assert_eq!(0b01100010, encoded[0]);

        let decoded = Pubrel::try_from(&encoded[..]).unwrap();
        assert_eq!(123_u16, decoded.packet_identifier);
        assert_eq!(0x00_u8, decoded.reason_code.into());
        assert!(decoded.properties.is_none());
    }

    #[test]
    fn encode_and_decode_with_properties() {
        let mut pubrel = Pubrel::new(6397, ReasonCode::PacketIdentifierNotFound).unwrap();
        let mut properties = PubrelProperties::default();
        properties.reason_string = Some("too lazy at the moment, apologies".into());
        properties.user_property.insert("options".into(), "none, really".into());
        pubrel.properties = Some(properties);

        let encoded: Vec<u8> = pubrel.into();
        let decoded = Pubrel::try_from(&encoded[..]).unwrap();
        assert_eq!(6397_u16, decoded.packet_identifier);
        assert_eq!(0x92_u8, decoded.reason_code.into());
        assert!(decoded.properties.is_some());
    }

    #[test]
    fn reason_code_validation() {
        assert!(Pubrel::new(123, ReasonCode::AdministrativeAction).is_err());
    }
}