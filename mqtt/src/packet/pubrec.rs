use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{types::{ReasonCode, MqttDataType}, error::MqttError, packet::Decodeable};

use super::MqttControlPacket;

/// `PUBREC` is the response to a `PUBLISH` that was sent with [crate::types::QoS::ExactlyOnce].
/// Must be followed by [`PUBREL`](crate::packet::Pubrel).
/// 
/// The sequence of messages for QoS 2 is as follows:
/// - `PUBLISH` -->
/// - `PUBREC` <--
/// - `PUBREL` -->
/// - `PUBCOMP` <-- 
#[derive(Debug)]
pub struct Pubrec {
    pub packet_identifier: u16,
    pub reason_code: ReasonCode,
    pub properties: Option<PubrecProperties>,
}

#[derive(Debug, MqttProperties)]
pub struct PubrecProperties {
    pub reason_string: Option<String>,
    pub user_property: HashMap<String, String>,
}

impl MqttControlPacket<'_> for Pubrec {
    fn packet_type() -> super::PacketType {
        super::PacketType::PUBREC
    }
}

/// Fixed first byte of the header
const FIRST_BYTE: u8 = 0b01010000;

impl Pubrec {

    pub fn new(packet_identifier: u16, reason_code: ReasonCode) -> Result<Self, MqttError> {
        Self::validate_reason_code(&reason_code)?;
        Ok(Self { packet_identifier, reason_code, properties: None })
    }

    fn validate_reason_code(reason_code: &ReasonCode) -> Result<(), MqttError> {
        match reason_code {
            ReasonCode::Success | 
            ReasonCode::NoMatchingSubscribers | 
            ReasonCode::UnspecifiedError |
            ReasonCode::ImplementationSpecificError |
            ReasonCode::NotAuthorized |
            ReasonCode::TopicNameInvalid |
            ReasonCode::PacketIdentifierInUse |
            ReasonCode::QuotaExceeded |
            ReasonCode::PayloadFormatInvalid => Ok(()),
            els => Err(MqttError::ProtocolError(format!("Invalid reason code [{}] for PUBREC", u8::from(*els)))),
        }
    }
}

impl From<Pubrec> for Vec<u8> {
    fn from(pubrec: Pubrec) -> Self {
        let mut result: Vec<u8> = Vec::new();

        result.push(FIRST_BYTE);
        super::push_be_u16(pubrec.packet_identifier, &mut result);
        result.push(pubrec.reason_code.into());

        match pubrec.properties {
            Some(props) => result.append(&mut props.into()),
            None => result.push(0),
        }

        super::calculate_and_insert_length(&mut result);

        result
    }
}

impl TryFrom<&[u8]> for Pubrec {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;

        match src[cursor] {
            FIRST_BYTE => cursor += 1,
            els => return Err(MqttError::MalformedPacket(format!("First byte is not a PUBREC one: {:b}", els)))
        }

        let remain_len = super::remaining_length(&src[cursor..])?;
        cursor += remain_len.encoded_len();

        let packet_identifier = super::u16_from_be_bytes(&src[cursor..])?;
        cursor += packet_identifier.encoded_len();

        let reason_code = ReasonCode::try_from(src[cursor])?;
        cursor += reason_code.encoded_len();
        
        let mut result = Self::new(packet_identifier, reason_code)?;
        result.properties = PubrecProperties::decode(&src[cursor..])?.value();
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn encode_and_decode() {
        let pubrec = Pubrec::new(123, ReasonCode::Success).unwrap();
        let encoded: Vec<u8> = pubrec.into();

        assert_eq!(0b01010000, encoded[0]);

        let decoded = Pubrec::try_from(&encoded[..]).unwrap();
        assert_eq!(123_u16, decoded.packet_identifier);
        assert_eq!(0x00_u8, decoded.reason_code.into());
        assert!(decoded.properties.is_none());
    }

    #[test]
    fn encode_and_decode_with_properties() {
        let mut pubrec = Pubrec::new(6397, ReasonCode::UnspecifiedError).unwrap();
        let mut properties = PubrecProperties::default();
        properties.reason_string = Some("too lazy at the moment, apologies".into());
        properties.user_property.insert("options".into(), "none, really".into());
        pubrec.properties = Some(properties);

        let encoded: Vec<u8> = pubrec.into();
        let decoded = Pubrec::try_from(&encoded[..]).unwrap();
        assert_eq!(6397_u16, decoded.packet_identifier);
        assert_eq!(0x80_u8, decoded.reason_code.into());
        assert!(decoded.properties.is_some());
    }

    #[test]
    fn reason_code_validation() {
        assert!(Pubrec::new(123, ReasonCode::AdministrativeAction).is_err());
    }
}