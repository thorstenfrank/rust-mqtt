use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{types::{ReasonCode, MqttDataType}, error::MqttError, packet::Decodeable};

/// `PUBACK` is the response to a `PUBLISH` that was sent with [crate::types::QoS::AtLeastOnce].
#[derive(Debug)]
pub struct Puback {
    pub packet_identifier: u16,
    pub reason_code: ReasonCode,
    pub properties: Option<PubackProperties>,
}

#[derive(Debug, MqttProperties)]
pub struct PubackProperties {
    pub reason_string: Option<String>,
    pub user_property: HashMap<String, String>,
}

/// Fixed first byte of the header
const FIRST_BYTE: u8 = 0b01000000;

impl Puback {

    pub fn new(packet_identifier: u16, reason_code: ReasonCode) -> Self {
        Self { packet_identifier, reason_code, properties: None }
    }
}

impl From<Puback> for Vec<u8> {
    fn from(puback: Puback) -> Self {
        let mut result: Vec<u8> = Vec::new();

        result.push(FIRST_BYTE);
        super::push_be_u16(puback.packet_identifier, &mut result);
        result.push(puback.reason_code.into());

        match puback.properties {
            Some(props) => result.append(&mut props.into()),
            None => result.push(0),
        }

        super::calculate_and_insert_length(&mut result);

        result
    }
}

impl TryFrom<&[u8]> for Puback {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;

        match src[cursor] {
            FIRST_BYTE => cursor += 1,
            els => return Err(MqttError::MalformedPacket(format!("First byte is not a PUBACK one: {:b}", els)))
        }

        let remain_len = super::remaining_length(&src[cursor..])?;
        cursor += remain_len.encoded_len();

        let packet_identifier = super::u16_from_be_bytes(&src[cursor..])?;
        cursor += packet_identifier.encoded_len();

        let reason_code = ReasonCode::try_from(src[cursor])?;
        cursor += reason_code.encoded_len();

        let properties = PubackProperties::decode(&src[cursor..])?.value();
        
        Ok(
            Self {
                packet_identifier,
                reason_code,
                properties,
            }
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn encode_and_decode() {
        let puback = Puback::new(123, ReasonCode::Success);
        let encoded: Vec<u8> = puback.into();
        let decoded = Puback::try_from(&encoded[..]).unwrap();
        assert_eq!(123_u16, decoded.packet_identifier);
        assert_eq!(0x00_u8, decoded.reason_code.into());
        assert!(decoded.properties.is_none());
    }

    #[test]
    fn encode_and_decode_with_properties() {
        let mut puback = Puback::new(6397, ReasonCode::UnspecifiedError);
        let mut properties = PubackProperties::default();
        properties.reason_string = Some("too lazy at the moment, apologies".into());
        properties.user_property.insert("options".into(), "none, really".into());
        puback.properties = Some(properties);

        let encoded: Vec<u8> = puback.into();
        let decoded = Puback::try_from(&encoded[..]).unwrap();
        assert_eq!(6397_u16, decoded.packet_identifier);
        assert_eq!(0x80_u8, decoded.reason_code.into());
        assert!(decoded.properties.is_some());
    }
}