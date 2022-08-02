use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{types::{ReasonCode, MqttDataType}, error::MqttError};

use super::{Decodeable, DecodingResult, MqttControlPacket};

#[derive(Debug)]
pub struct Unsuback {
    pub packet_identifier: u16,
    pub properties: Option<UnsubackProperties>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Debug, MqttProperties)]
pub struct UnsubackProperties {
    pub reason_string: Option<String>,
    pub user_property: HashMap<String, String>,
}

const FIRST_BYTE: u8 = 0b10110000;

impl MqttControlPacket<'_> for Unsuback {
    fn packet_type() -> super::PacketType {
        super::PacketType::UNSUBACK
    }
}

impl From<Unsuback> for Vec<u8> {
    fn from(unsuback: Unsuback) -> Self {
        let mut result = Vec::new();
        result.push(FIRST_BYTE);
        super::push_be_u16(unsuback.packet_identifier, &mut result);
        match unsuback.properties {
            Some(props) => result.append(&mut props.into()),
            None => result.push(0),
        }
        for code in unsuback.reason_codes {
            result.push(code.into());
        }

        super::calculate_and_insert_length(&mut result);

        result
    }
}

impl TryFrom<&[u8]> for Unsuback {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;
        match src[cursor] {
            FIRST_BYTE => cursor += 1,
            els => return Err(MqttError::MalformedPacket(format!("First byte is not a UNSUBACK one: {:b}", els)))
        }

        let remain_len = super::remaining_length(&src[cursor..])?;
        cursor += remain_len.encoded_len();
        let cursor_stop = cursor + remain_len.value as usize;

        let packet_identifier = super::u16_from_be_bytes(&src[cursor..])?;
        cursor += packet_identifier.encoded_len();

        let props_result: DecodingResult<UnsubackProperties> = UnsubackProperties::decode(&src[cursor..])?;
        let properties = props_result.value;
        cursor += props_result.bytes_read;

        let mut reason_codes = Vec::new();

        while cursor < cursor_stop {
            reason_codes.push(ReasonCode::try_from(src[cursor])?);
            cursor += 1;
        }        

        Ok(Self {
            packet_identifier,
            properties,
            reason_codes,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn encode_and_decode() {
        let unsuback = Unsuback { packet_identifier: 872, properties: None, reason_codes: vec![ReasonCode::Success, ReasonCode::NoSubscriptionExisted] };
        let encoded: Vec<u8> = unsuback.into();
        let decoded = Unsuback::try_from(&encoded[..]).unwrap();
        assert_eq!(872, decoded.packet_identifier);
        assert_eq!(2, decoded.reason_codes.len());
    }
}