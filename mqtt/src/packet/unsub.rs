use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{error::MqttError, types::{UTF8String, MqttDataType}};

use super::{Decodeable, DecodingResult, MqttControlPacket};

#[derive(Debug)]
pub struct Unsubscribe {
    pub packet_identifier: u16,
    pub properties: Option<UnsubscribeProperties>,
    pub topic_filter: Vec<String>,
}

#[derive(Debug, MqttProperties)]
pub struct UnsubscribeProperties{
    pub user_property: HashMap<String, String>,
}

const FIRST_BYTE: u8 = 0b10100010;

impl MqttControlPacket<'_> for Unsubscribe {
    fn packet_type() -> super::PacketType {
        super::PacketType::UNSUBSCRIBE
    }
}

impl From<Unsubscribe> for Vec<u8> {
    fn from(unsub: Unsubscribe) -> Self {
        let mut result = Vec::new();
        result.push(FIRST_BYTE);
        super::push_be_u16(unsub.packet_identifier, &mut result);
        match unsub.properties {
            Some(props) => result.append(&mut props.into()),
            None => result.push(0),
        }
        for filter in unsub.topic_filter {
            result.append(&mut UTF8String::from(filter).into())
        }
        super::calculate_and_insert_length(&mut result);

        result
    }
}

impl TryFrom<&[u8]> for Unsubscribe {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;
        match src[cursor] {
            FIRST_BYTE => cursor += 1,
            els => return Err(MqttError::MalformedPacket(format!("First byte is not a UNSUBSCRIBE one: {:b}", els)))
        }

        let remain_len = super::remaining_length(&src[cursor..])?;
        cursor += remain_len.encoded_len();
        let cursor_stop = cursor + remain_len.value as usize;

        let packet_identifier = super::u16_from_be_bytes(&src[cursor..])?;
        cursor += packet_identifier.encoded_len();

        let props_result: DecodingResult<UnsubscribeProperties> = UnsubscribeProperties::decode(&src[cursor..])?;
        let properties = props_result.value;
        cursor += props_result.bytes_read;

        let mut topic_filter = Vec::new();

        while cursor < cursor_stop {
            let filter = UTF8String::try_from(&src[cursor..])?;
            cursor += filter.encoded_len();

            if let Some(v) = filter.value {
                topic_filter.push(v);
            }
        }

        Ok(Self {
            packet_identifier,
            properties,
            topic_filter,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Unsubscribe;


    #[test]
    fn encode_and_decode() {
        let topic_filter: Vec<String> = vec!["/some/topic".into()];
        let unsub = Unsubscribe {
            packet_identifier: 1782,
            properties: None,
            topic_filter,
        };
        let encoded: Vec<u8> = unsub.into();
        let decoded = Unsubscribe::try_from(&encoded[..]).unwrap();
        assert_eq!(1782, decoded.packet_identifier);
    }
}