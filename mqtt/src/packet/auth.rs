use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{types::ReasonCode, error::MqttError};

use super::{MqttControlPacket, Decodeable, MqttDataType};

#[derive(Debug)]
pub struct Auth {
    pub reason_code: ReasonCode,
    pub properties: Option<AuthProperties>
}

#[derive(Debug, MqttProperties)]
pub struct AuthProperties {
    pub authentication_method: Option<String>,
    pub authentication_data: Option<Vec<u8>>,
    pub reason_string: Option<String>,
    pub user_property: HashMap<String, String>,
}

const FIRST_BYTE: u8 = 0b11110000;

impl MqttControlPacket<'_> for Auth {
    fn packet_type() -> super::PacketType {
        super::PacketType::AUTH
    }
}

impl From<Auth> for Vec<u8> {
    fn from(auth: Auth) -> Self {
        let mut result = Vec::new();
        result.push(FIRST_BYTE);

        if auth.reason_code != ReasonCode::Success || auth.properties.is_some() {
            result.push(auth.reason_code.into());
            match auth.properties {
                Some(props) => result.append(&mut props.into()),
                None => result.push(0),
            }
        }

        super::calculate_and_insert_length(&mut result);

        result
    }
}

impl TryFrom<&[u8]> for Auth {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;

        match src[cursor] {
            FIRST_BYTE => cursor += 1,
            els => return Err(MqttError::MalformedPacket(format!("First byte is not an AUTH one: {:b}", els)))
        }

        let remain_len = super::remaining_length(&src[cursor..])?;
        cursor += remain_len.encoded_len();
        
        let (reason_code, properties) = match remain_len.value {
            0 => (ReasonCode::Success, None),
            _ => {
                let reason_code = ReasonCode::try_from(src[cursor])?;
                cursor += 1;
                let props_res = AuthProperties::decode(&src[cursor..])?;
                (reason_code, props_res.value)
            }
        };

        Ok(Self{
            reason_code,
            properties,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_and_decode() {
        let auth = Auth { reason_code: ReasonCode::Success, properties: None };
        let encoded: Vec<u8> = auth.into();
        assert_eq!(2, encoded.len());
        let decoded = Auth::try_from(&encoded[..]).unwrap();
        assert_eq!(ReasonCode::Success, decoded.reason_code);
    }
}