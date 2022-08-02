use std::collections::HashMap;

use mqtt_derive::MqttProperties;

use crate::{types::{QoS, VariableByteInteger, UTF8String, MqttDataType}, error::MqttError};

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

/// Packet Type 1000 | Reserved 0000
const FIRST_BYTE: u8 = 0b10000010;

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

#[cfg(test)]
mod tests {
    use super::*;

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