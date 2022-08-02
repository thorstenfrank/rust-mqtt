use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{types::{ReasonCode, MqttDataType}, error::MqttError, packet::Decodeable};

/// `PUBCOMP` is the final message in the flow initiated with `PUBLISH` sent with [crate::types::QoS::ExactlyOnce].
/// 
/// The sequence of messages for QoS 2 is as follows:
/// - `PUBLISH` -->
/// - `PUBREC` <--
/// - `PUBREL` -->
/// - `PUBCOMP` <-- 
#[derive(Debug)]
pub struct Pubcomp {
    pub packet_identifier: u16,
    pub reason_code: ReasonCode,
    pub properties: Option<PubcompProperties>,
}

#[derive(Debug, MqttProperties)]
pub struct PubcompProperties {
    pub reason_string: Option<String>,
    pub user_property: HashMap<String, String>,
}

/// Fixed first byte of the header
const FIRST_BYTE: u8 = 0b01110000;

impl Pubcomp {

    pub fn new(packet_identifier: u16, reason_code: ReasonCode) -> Result<Self, MqttError> {
        Self::validate_reason_code(&reason_code)?;
        Ok(Self { packet_identifier, reason_code, properties: None })
    }

    fn validate_reason_code(reason_code: &ReasonCode) -> Result<(), MqttError> {
        match reason_code {
            ReasonCode::Success | 
            ReasonCode::PacketIdentifierNotFound => Ok(()),
            els => Err(MqttError::ProtocolError(format!("Invalid reason code [{}] for PUBCOMP", u8::from(*els)))),
        }
    }
}

impl From<Pubcomp> for Vec<u8> {
    fn from(pubcomp: Pubcomp) -> Self {
        let mut result: Vec<u8> = Vec::new();

        result.push(FIRST_BYTE);
        super::push_be_u16(pubcomp.packet_identifier, &mut result);
        result.push(pubcomp.reason_code.into());

        match pubcomp.properties {
            Some(props) => result.append(&mut props.into()),
            None => result.push(0),
        }

        super::calculate_and_insert_length(&mut result);

        result
    }
}

impl TryFrom<&[u8]> for Pubcomp {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;

        match src[cursor] {
            FIRST_BYTE => cursor += 1,
            els => return Err(MqttError::MalformedPacket(format!("First byte is not a PUBCOMP one: {:b}", els)))
        }

        let remain_len = super::remaining_length(&src[cursor..])?;
        cursor += remain_len.encoded_len();

        let packet_identifier = super::u16_from_be_bytes(&src[cursor..])?;
        cursor += packet_identifier.encoded_len();

        let reason_code = ReasonCode::try_from(src[cursor])?;
        cursor += reason_code.encoded_len();
        
        let mut result = Self::new(packet_identifier, reason_code)?;
        result.properties = PubcompProperties::decode(&src[cursor..])?.value();
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn encode_and_decode() {
        let pubcomp = Pubcomp::new(123, ReasonCode::Success).unwrap();
        // [112, 4, 0, 123, 0, 0]
        let encoded: Vec<u8> = pubcomp.into();
        assert_eq!(0b01110000, encoded[0]);

        let decoded = Pubcomp::try_from(&encoded[..]).unwrap();
        assert_eq!(123_u16, decoded.packet_identifier);
        assert_eq!(0x00_u8, decoded.reason_code.into());
        assert!(decoded.properties.is_none());
    }

    #[test]
    fn encode_and_decode_with_properties() {
        let mut pubcomp = Pubcomp::new(6397, ReasonCode::PacketIdentifierNotFound).unwrap();
        let mut properties = PubcompProperties::default();
        properties.reason_string = Some("too lazy at the moment, apologies".into());
        properties.user_property.insert("options".into(), "none, really".into());
        pubcomp.properties = Some(properties);

        let encoded: Vec<u8> = pubcomp.into();
        let decoded = Pubcomp::try_from(&encoded[..]).unwrap();
        assert_eq!(6397_u16, decoded.packet_identifier);
        assert_eq!(0x92_u8, decoded.reason_code.into());
        assert!(decoded.properties.is_some());
    }

    #[test]
    fn reason_code_validation() {
        assert!(Pubcomp::new(123, ReasonCode::AdministrativeAction).is_err());
    }
}