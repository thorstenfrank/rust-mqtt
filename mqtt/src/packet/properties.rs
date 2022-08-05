//! This is internal API - types and utils to work with properties, which can occur in almost any MQTT control packet
//! as well the the last will. Since all packets work on the same (sub-) set of these properties, they're collected 
//! here to allow packets to work with them efficiently.

use crate::{
    error::MqttError,
    types::{BinaryData, MqttDataType, UTF8String, VariableByteInteger, UTF8StringPair},
};

use super::{encode_and_append, u16_from_be_bytes, u32_from_be_bytes};

/// Numeric IDs.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PropertyIdentifier {
    PayloadFormatIndicator = 1,
    MessageExpiryInterval = 2,
    ContentType = 3,
    ResponseTopic = 8,
    CorrelationData = 9,
    SubscriptionIdentifier = 11,
    SessionExpiryInterval = 17,
    AssignedClientIdentifier = 18,
    ServerKeepAlive = 19,
    AuthenticationMethod = 21,
    AuthenticationData = 22,
    RequestProblemInformation = 23,
    WillDelayInterval = 24,
    RequestResponseInformation = 25,
    ResponseInformation = 26,
    ServerReference = 28,
    ReasonString = 31,
    ReceiveMaximum = 33,
    TopicAliasMaximum = 34,
    TopicAlias = 35,
    MaximumQos = 36,
    RetainAvailable = 37,
    UserProperty = 38,
    MaximumPacketSize = 39,
    WildcardSubscriptionAvailable = 40,
    SubscriptionIdentifierAvailable = 41,
    SharedSubscriptionAvailable = 42,
}

/// MQTT control packets may include optional properties as part of the variable header.
#[derive(Debug)]
pub struct MqttProperty {
    /// One of the defined IDs.
    pub identifier: PropertyIdentifier,

    /// A "container" for holding the actual data.
    pub value: DataRepresentation,
}

/// We need this enum as a wrapper around the actual datatypes so we can get some type of polymorphism
/// going without having to manually unscrew the lid off the heap and hardwire bits just to have the compiler scream at us.
#[derive(Debug, PartialEq)]
pub enum DataRepresentation {
    /// Single byte value
    Byte(u8),

    /// Unsigned 16-bit integer
    TwoByteInt(u16),

    /// Unsigned 32-bit integer
    FourByteInt(u32),

    /// Variable Byte Integer, see `MQTT 1.5.5`
    VariByteInt(VariableByteInteger),

    /// UTF-8 String, with 16-bit length info.
    UTF8(UTF8String),

    /// Key-value pair
    UTF8Pair(UTF8StringPair),

    /// Binary data, with preceding 16-bit length info.
    BinaryData(BinaryData),
}

/// Parses the supplied byte slice and calls the supplied callback function for each parsed [`MqttProperty`].
/// The first byte(s) of the `src` slice *must* be a variable byte integer that determines how many of the following 
/// bytes represent data that can be parsed into 0 to n properties.
/// The result will contain the number of bytes that were used during parsing. If parsing was successful, then the min
/// length read will be `1` - the byte to represent the length of `0` properties.
pub fn parse_properties<F>(src: &[u8], mut f: F) -> Result<usize, MqttError> 
where
    F: FnMut(MqttProperty) -> Result<(), MqttError>
{
    if src.len() == 0 {
        return Ok(0)
    }
    
    let properties_length = VariableByteInteger::try_from(src)?;
    let length: usize = properties_length.value.try_into().unwrap();

    let remain = &src[properties_length.encoded_len()..];
    let mut cursor = 0;

    while cursor < length {
        let identifier = PropertyIdentifier::try_from(&remain[cursor])?;
        cursor += 1;

        let value = match identifier {
            PropertyIdentifier::PayloadFormatIndicator | 
            PropertyIdentifier::RequestProblemInformation | 
            PropertyIdentifier::RequestResponseInformation | 
            PropertyIdentifier::MaximumQos | 
            PropertyIdentifier::RetainAvailable | 
            PropertyIdentifier::WildcardSubscriptionAvailable |
            PropertyIdentifier::SubscriptionIdentifierAvailable |
            PropertyIdentifier::SharedSubscriptionAvailable => {
                DataRepresentation::Byte(remain[cursor])
            },
            PropertyIdentifier::ServerKeepAlive |
            PropertyIdentifier::ReceiveMaximum |
            PropertyIdentifier::TopicAliasMaximum |
            PropertyIdentifier::TopicAlias => {
                DataRepresentation::decode_as_u16(&remain[cursor..])?
            },
            PropertyIdentifier::MessageExpiryInterval |
            PropertyIdentifier::SessionExpiryInterval |
            PropertyIdentifier::MaximumPacketSize |
            PropertyIdentifier::WillDelayInterval => {
                DataRepresentation::decode_as_u32(&remain[cursor..])?
            },
            PropertyIdentifier::ContentType |
            PropertyIdentifier::ResponseTopic |
            PropertyIdentifier::AssignedClientIdentifier |
            PropertyIdentifier::AuthenticationMethod |
            PropertyIdentifier::ResponseInformation |
            PropertyIdentifier::ServerReference |
            PropertyIdentifier::ReasonString => {
                DataRepresentation::UTF8(UTF8String::try_from(&remain[cursor..])?)
            },
            PropertyIdentifier::CorrelationData |
            PropertyIdentifier::AuthenticationData => {
                DataRepresentation::BinaryData(BinaryData::try_from(&remain[cursor..])?)
            },
            PropertyIdentifier::SubscriptionIdentifier => {
                DataRepresentation::VariByteInt(VariableByteInteger::try_from(&remain[cursor..])?)
            },
            PropertyIdentifier::UserProperty => {
                DataRepresentation::UTF8Pair(UTF8StringPair::try_from(&remain[cursor..])?)
            }
        };

        cursor += value.encoded_len();
        f(MqttProperty { identifier, value })?;
    }

    Ok(properties_length.encoded_len() + cursor)
}

pub fn encode_and_append_property(identifier: PropertyIdentifier, value: DataRepresentation, target: &mut Vec<u8>) -> u32 {
    // yeah, this isn't super safe...
    let len = value.encoded_len() as u32 + 1;
    let mut property: Vec<u8> = MqttProperty { identifier, value }.into();
    target.append(&mut property);
    len
}

impl TryFrom<&u8> for PropertyIdentifier {
    type Error = MqttError;
    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        let result: PropertyIdentifier = match value {
            1 => Self::PayloadFormatIndicator,
            2 => Self::MessageExpiryInterval,
            3 => Self::ContentType,
            8 => Self::ResponseTopic,
            9 => Self::CorrelationData,
            11 => Self::SubscriptionIdentifier,
            17 => Self::SessionExpiryInterval,
            18 => Self::AssignedClientIdentifier,
            19 => Self::ServerKeepAlive,
            21 => Self::AuthenticationMethod,
            22 => Self::AuthenticationData,
            23 => Self::RequestProblemInformation,
            24 => Self::WillDelayInterval,
            25 => Self::RequestResponseInformation,
            26 => Self::ResponseInformation,
            28 => Self::ServerReference,
            31 => Self::ReasonString,
            33 => Self::ReceiveMaximum,
            34 => Self::TopicAliasMaximum,
            35 => Self::TopicAlias,
            36 => Self::MaximumQos,
            37 => Self::RetainAvailable,
            38 => Self::UserProperty,
            39 => Self::MaximumPacketSize,
            40 => Self::WildcardSubscriptionAvailable,
            41 => Self::SubscriptionIdentifierAvailable,
            42 => Self::SharedSubscriptionAvailable,
            _ => {
                return Err(MqttError::Message(format!(
                    "Unknown property identifier: {}",
                    value
                )))
            }
        };

        Ok(result)
    }
}

/// FIXME we really should introduce a separate trait for defining encodeable elements, not "abuse" the MqttDataType
impl MqttDataType for PropertyIdentifier {
    fn encoded_len(&self) -> usize {
        1 // right now all values are < 128. will have to change to VBI eventually
    }
}

impl DataRepresentation {

    /// Returns [`DataRepresentation::TwoByteInt`]
    fn decode_as_u16(src: &[u8]) -> Result<Self, MqttError> {
        Ok(Self::TwoByteInt(u16_from_be_bytes(&src)?))
    }

    /// Returns [`DataRepresentation::FourByteInt`]
    fn decode_as_u32(src: &[u8]) -> Result<Self, MqttError> {
        Ok(Self::FourByteInt(u32_from_be_bytes(&src)?))
    }
}

impl MqttDataType for DataRepresentation {
    fn encoded_len(&self) -> usize {
        match self {
            DataRepresentation::Byte(v) => v.encoded_len(),
            DataRepresentation::TwoByteInt(v) => v.encoded_len(),
            DataRepresentation::FourByteInt(v) => v.encoded_len(),
            DataRepresentation::VariByteInt(v) => v.encoded_len(),
            DataRepresentation::UTF8(v) => v.encoded_len(),
            DataRepresentation::UTF8Pair(v) => v.encoded_len(),
            DataRepresentation::BinaryData(v) => v.encoded_len(),
        }
    }
}

impl TryInto<bool> for DataRepresentation {
    type Error = MqttError;

    fn try_into(self) -> Result<bool, Self::Error> {
        if let DataRepresentation::Byte(v) = self {
            match v {
                0 => return Ok(false),
                1 => return Ok(true),
                _=> return Err(MqttError::ProtocolError(format!("illegal value for [shared subscription available]: {}", v))),
            }
        }

        Err(MqttError::ProtocolError(format!("only DataRepresentation::Byte can be converted to bool. Is: {:?}", self)))

    }
}

impl MqttDataType for MqttProperty {
    fn encoded_len(&self) -> usize {
        self.identifier.encoded_len() + self.value.encoded_len()
    }
}

impl Into<Vec<u8>> for MqttProperty {
    fn into(self) -> Vec<u8> {
        let mut result = Vec::new();

        // this works for now, because all IDs have a numeric value < 127
        // technically, this should be a Variable Byte Integer, though
        result.push(self.identifier as u8);

        match self.value {
            DataRepresentation::Byte(b) => result.push(b),
            DataRepresentation::TwoByteInt(i) => super::push_be_u16(i, &mut result),
            DataRepresentation::FourByteInt(i) => super::push_be_u32(i, &mut result),
            DataRepresentation::VariByteInt(v) => encode_and_append(v, &mut result),
            DataRepresentation::UTF8(v) => encode_and_append(v, &mut result),
            DataRepresentation::UTF8Pair(v) => encode_and_append(v, &mut result),
            DataRepresentation::BinaryData(v) => encode_and_append(v, &mut result),
        }
        
        result
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn encode_property() {
        test_encode(PropertyIdentifier::PayloadFormatIndicator, DataRepresentation::Byte(1), vec![1, 1]);
        test_encode(PropertyIdentifier::MessageExpiryInterval, DataRepresentation::FourByteInt(600), vec![2, 0, 0, 2, 88]);
        test_encode(PropertyIdentifier::AuthenticationMethod, DataRepresentation::UTF8(UTF8String::from("basic")), vec![21, 0, 5, 98, 97, 115, 105, 99]);
        test_encode(
            PropertyIdentifier::AuthenticationData, 
            DataRepresentation::BinaryData(BinaryData::new(vec![2, 4, 6, 8, 10, 1, 3, 5, 7, 9]).unwrap()), 
            vec![22, 0, 10, 2, 4, 6, 8, 10, 1, 3, 5, 7, 9]
        );
    }
    
    fn test_encode(identifier: PropertyIdentifier, value: DataRepresentation, expected: Vec<u8>) {
        let prop = MqttProperty { identifier, value };
        let encoded: Vec<u8> = prop.into();
        assert_eq!(expected, encoded);
    }
}