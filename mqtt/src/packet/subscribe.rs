use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{types::{QoS, VariableByteInteger, UTF8String, MqttDataType}, error::MqttError};
use super::{Decodeable, DecodingResult, MqttControlPacket};

/// A `SUBSCRIBE` packet from a client is the prerequisite to receiving messages through [crate::packet::Publish].
#[derive(Debug)]
pub struct Subscribe {
    pub packet_identifier: u16,
    pub properties: Option<SubscribeProperties>,
    pub topic_filter: Vec<TopicFilter>,
}

#[derive(Debug, MqttProperties)]
pub struct SubscribeProperties {
    pub subscription_identifier: Option<VariableByteInteger>,
    pub user_property: HashMap<String, String>,
}

#[derive(Debug)]
pub struct TopicFilter {
    pub filter: String,
    /// Defaults to [crate::types::QoS::AtLeastOnce]
    pub maximum_qos: QoS,
    /// Default: `false`
    pub no_local: bool,
    /// Default: `false`
    pub retain_as_published: bool,
    /// Defaults to `0`
    pub retain_handling: RetainHandling,
}

/// Defines how retained messages are to be dealt with by the server.
#[derive(Debug, PartialEq, PartialOrd)]
pub enum RetainHandling {
    /// Sends retained messages directly on subscribe
    OnSubscribe = 0,
    /// Send only if this subscription does not yet exist
    NewSubOnly = 1,
    /// Self-explanatory
    Never = 2,
}

impl MqttControlPacket<'_> for Subscribe {
    fn packet_type() -> super::PacketType {
        super::PacketType::SUBSCRIBE
    }
}

/// Packet Type 1000 | Reserved 0000
const FIRST_BYTE: u8 = 0b10000010;

impl From<Subscribe> for Vec<u8> {
    fn from(subscribe: Subscribe) -> Self {
        let mut result = Vec::new();
        result.push(FIRST_BYTE);
        super::push_be_u16(subscribe.packet_identifier, &mut result);
        match subscribe.properties {
            Some(props) => result.append(&mut props.into()),
            None => result.push(0),
        }
        for filter in subscribe.topic_filter {
            result.append(&mut filter.into())
        }

        super::calculate_and_insert_length(&mut result);

        result
    }
}

impl TryFrom<&[u8]> for Subscribe {
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

        let props_result: DecodingResult<SubscribeProperties> = SubscribeProperties::decode(&src[cursor..])?;
        let properties = props_result.value;
        cursor += props_result.bytes_read;

        let mut topic_filter = Vec::new();

        while cursor < cursor_stop {
            let filter = TopicFilter::try_from(&src[cursor..])?;
            cursor += filter.encoded_len();
            topic_filter.push(filter);
        }

        Ok(Self {
            packet_identifier,
            properties,
            topic_filter,
        })
    }
}

impl TopicFilter {

    /// Creates a new filter with default options.
    pub fn new(filter: String) -> Self {
        TopicFilter {
            filter,
            maximum_qos: QoS::AtMostOnce,
            no_local: false,
            retain_as_published: false,
            retain_handling: RetainHandling::OnSubscribe,
        }
    }
}

impl From<TopicFilter> for Vec<u8> {
    fn from(filter: TopicFilter) -> Self {
        let mut res = Vec::new();
        res.append(&mut UTF8String::from(filter.filter).into());

        // setting bits 0 and 1 directly is just easier
        let mut options: u8 = match filter.maximum_qos {
            QoS::AtMostOnce => 0,
            QoS::AtLeastOnce => 0b00000001,
            QoS::ExactlyOnce => 0b00000010,
        };

        if filter.no_local {
            options |= 0b00000100;
        }

        if filter.retain_as_published {
            options |= 0b00001000;
        }

        options = match filter.retain_handling {
            RetainHandling::OnSubscribe => options,//nothing to do here,
            RetainHandling::NewSubOnly => options | 0b00010000,
            RetainHandling::Never => options | 0b00100000,
        };

        res.push(options);

        res
    }
}

impl TryFrom<&[u8]> for TopicFilter {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let filter_decoded = UTF8String::try_from(&src[..])?;
        let filter = match &filter_decoded.value {
            Some(f) => f.clone(),
            None => return Err(MqttError::ProtocolError("Topic Filter missing".into())),
        };

        let options = src[filter_decoded.encoded_len()];
        let maximum_qos = QoS::try_from(options & 0b00000011)?;
        let no_local = match (options & 0b00000100) >> 2 {
            0 => false,
            1 => true,
            els => return Err(MqttError::Message(format!("Invalid value for bool: {:?}", els)))
        };

        let retain_as_published = match (options & 0b00001000) >> 3 {
            0 => false,
            1 => true,
            els => return Err(MqttError::Message(format!("Invalid value for bool: {:?}", els)))
        };

        let retain_handling = match (options & 0b00110000) >> 4 {
            0 => RetainHandling::OnSubscribe,
            1 => RetainHandling::NewSubOnly,
            2 => RetainHandling::Never,
            els => return Err(MqttError::ProtocolError(format!("Illegal value for [retain handling]: {:?}", els)))
        };

        Ok(Self {
            filter,
            maximum_qos,
            no_local,
            retain_as_published,
            retain_handling,
        })
    }
}

impl MqttDataType for TopicFilter {
    fn encoded_len(&self) -> usize {
        // number of bytes of the string value, plus 2 bytes for the length field plus
        // 1 byte for the options
        self.filter.len() + 2 + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_and_decode() {
        let topic_filter: Vec<TopicFilter> = vec![TopicFilter::new("/some/topic".into())];
        let subscribe = Subscribe{
            packet_identifier: 637,
            properties: None,
            topic_filter,
        };

        let encoded: Vec<u8> = subscribe.into();

        let decoded = Subscribe::try_from(&encoded[..]).unwrap();
        assert_eq!("/some/topic".to_string(), decoded.topic_filter[0].filter)
    }

    #[test]
    fn encode_decode_topic_filter() {
        let f1 = TopicFilter::new("/some/topic".into());
        let e1: Vec<u8> = f1.into();
        assert_eq!(e1, vec![0, 11, 47,115,111,109,101,47,116,111,112,105,99,0]);

        let d1 = TopicFilter::try_from(&e1[..]).unwrap();
        assert_eq!(d1.filter, "/some/topic".to_string());
        assert_eq!(d1.maximum_qos, QoS::AtMostOnce);
        assert_eq!(d1.no_local, false);
        assert_eq!(d1.retain_as_published, false);
        assert_eq!(d1.retain_handling, RetainHandling::OnSubscribe);

        let mut f2 = TopicFilter::new("/some/topic".into());
        f2.maximum_qos = QoS::AtLeastOnce;
        f2.no_local = true;
        f2.retain_as_published = true;
        f2.retain_handling = RetainHandling::Never;
        let e2: Vec<u8> = f2.into();
        assert_eq!(e2, vec![0, 11, 47,115,111,109,101,47,116,111,112,105,99,45]);

        let d2 = TopicFilter::try_from(&e2[..]).unwrap();
        assert_eq!(d2.filter, "/some/topic".to_string());
        assert_eq!(d2.maximum_qos, QoS::AtLeastOnce);
        assert_eq!(d2.no_local, true);
        assert_eq!(d2.retain_as_published, true);
        assert_eq!(d2.retain_handling, RetainHandling::Never);
    }
}