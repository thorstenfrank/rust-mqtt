use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{types::{ReasonCode, MqttDataType}, error::MqttError};
use super::{Decodeable, DecodingResult};

/// A `SUBACK` packet is sent by the Server to the Client to confirm receipt and processing of a `SUBSCRIBE` packet.
/// 
/// The payload ontains a list of [Reason Codes](crate::types::ReasonCode) that specify the maximum QoS level that was
/// granted or the error which was found for each Subscription that was requested by the 
/// [`SUBSCRIBE`](crate::packet::Subscribe).
#[derive(Debug)]
pub struct Suback {
    pub packet_identifier: u16,
    pub properties: Option<SubackProperties>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Debug, MqttProperties)]
pub struct SubackProperties {
    reason_string: Option<String>,
    user_property: HashMap<String, String>,
}

const FIRST_BYTE: u8 = 0b10010000;

impl From<Suback> for Vec<u8> {
    fn from(suback: Suback) -> Self {
        let mut result = Vec::new();
        result.push(FIRST_BYTE);
        super::push_be_u16(suback.packet_identifier, &mut result);
        match suback.properties {
            Some(p) => result.append(&mut p.into()),
            None => result.push(0),
        }

        for c in suback.reason_codes {
            result.push(c.into())
        }

        super::calculate_and_insert_length(&mut result);

        result
    }
}

impl TryFrom<&[u8]> for Suback {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;
        match src[cursor] {
            FIRST_BYTE => cursor += 1,
            els => return Err(MqttError::MalformedPacket(format!("First byte is not a SUBSCRIBE one: {:b}", els)))
        }

        let remain_len = super::remaining_length(&src[cursor..])?;
        cursor += remain_len.encoded_len();
        let cursor_stop = cursor + remain_len.value as usize;

        let packet_identifier = super::u16_from_be_bytes(&src[cursor..])?;
        cursor += packet_identifier.encoded_len();

        let props_result: DecodingResult<SubackProperties> = SubackProperties::decode(&src[cursor..])?;
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
        let suback = Suback{ packet_identifier: 2345, properties: None, reason_codes: vec![ReasonCode::Success] };
        let encoded: Vec<u8> = suback.into();
        assert_eq!(encoded, vec![144, 4, 9, 41, 0, 0]);
        let decoded = Suback::try_from(&encoded[..]).unwrap();
        assert_eq!(2345, decoded.packet_identifier);
        assert_eq!(ReasonCode::Success, decoded.reason_codes[0]);
    }
}