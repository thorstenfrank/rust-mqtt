use std::fmt::Display;

use crate::error::MqttError;

use super::MqttDataType;

/// A String with a max length of 65,535 bytes (not characters!).
/// The encoded value also includes the length in two bytes.
/// See [MQTT-1.5.4](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901010).
#[derive(Debug, PartialEq, Hash, Eq)]
pub struct UTF8String {
    pub value: Option<String>,
}

/// Just two [UTF8String]s in a row. 
/// See [the spec](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901013).
#[derive(Debug, PartialEq)]
pub struct UTF8StringPair {
    pub key: UTF8String,
    pub value: UTF8String,
}

impl MqttDataType for UTF8String {
    fn encoded_len(&self) -> usize {
        let mut len = 2;
        if let Some(s) = &self.value {
            len += s.len()
        }
        len
    }
}

impl UTF8String {

    const LENGTH_FIELD_SIZE: usize = 2;

    /// Creates a new, empty UTF8String that will consist in binary form only as a byte representing a length of 0. 
    pub fn new() -> Self {
        UTF8String { value: None }
    }
}

/// FIXME this should be changed to TryFrom to handle length overruns
impl From<String> for UTF8String {
    fn from(val: String) -> Self {
        UTF8String { value: Some(val) }
    }
}

/// FIXME this should be changed to TryFrom to handle length overruns
impl From<&str> for UTF8String {
    fn from(val: &str) -> Self {
        UTF8String { value: Some(val.into()) }
    }
}

impl From<UTF8String> for Vec<u8> {
    fn from(src: UTF8String) -> Self {
        match src.value {
            Some(utf8) => {
                let bytes = utf8.as_bytes();
                let mut result = Vec::new();
                let length = bytes.len() as u16;
                for b in length.to_be_bytes() {
                    result.push(b)
                }

                for b in bytes {
                    result.push(*b)
                }
        
                result                
            },
            None => vec![0, 0],
        }
    }
}

impl From<UTF8String> for String {
    /// Returns an empty `String` if [UTF8String.value] is `None`.
    fn from(src: UTF8String) -> Self {
        match src.value {
            Some(s) => s,
            None => String::new(),
        }
    }
}

impl TryFrom<&[u8]> for UTF8String {

    type Error = MqttError;
    
    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let (len_slice, value) = src.split_at(UTF8String::LENGTH_FIELD_SIZE);
        let length:usize = u16::from_be_bytes([len_slice[0], len_slice[1]]).into();
        
        match String::from_utf8(value[..length].to_vec()) {
            Ok(s) => Ok(UTF8String::from(s)),
            Err(e) => Err(MqttError::Message(format!("Error decoding bytes to String: {:?}", e))),
        }
    }
}

impl PartialEq<UTF8String> for String {
    fn eq(&self, other: &UTF8String) -> bool {
        match &other.value {
            Some(v) => self.eq(v),
            None => false,
        }
    }
}

impl Display for UTF8String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Some(v) => v.fmt(f),
            None => Ok(()),
        }
    }
}

impl UTF8StringPair {
    pub fn new(key: String, value: String) -> Self {
        UTF8StringPair { key: UTF8String::from(key), value: UTF8String::from(value) }
    }
}

impl MqttDataType for UTF8StringPair {
    
    fn encoded_len(&self) -> usize {
        self.key.encoded_len() + self.value.encoded_len()
    }

}

impl From<UTF8StringPair> for Vec<u8> {
    fn from(src: UTF8StringPair) -> Self {
        let mut result: Vec<u8> = src.key.into();
        let mut val: Vec<u8> = src.value.into();
        result.append(&mut val);
        result
    }
}

impl TryFrom<&[u8]> for UTF8StringPair {
    type Error = MqttError;

    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let key = UTF8String::try_from(&src[..])?;
        let value = UTF8String::try_from(&src[key.encoded_len()..])?;
        
        Ok(UTF8StringPair { key, value })
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn encode_utf8() {
        let utf8 = UTF8String::from("MQTT");
        assert_eq!(6, utf8.encoded_len());
        
        let expect: Vec<u8> = vec![0, 4, 77, 81, 84, 84];
        let actual: Vec<u8> = utf8.into();
        assert_eq!(expect, actual);
    }

    #[test]
    fn encode_empty() {
        let utf8 = UTF8String::new();
        let expect: Vec<u8> = vec![0, 0];
        let actual: Vec<u8> = utf8.into();
        assert_eq!(expect, actual);
    }

    #[test]
    fn decode_utf8() {
        let source: Vec<u8> = vec![0, 4, 77, 81, 84, 84];
        let expect = UTF8String::from("MQTT");
        let actual = UTF8String::try_from(source.as_slice()).unwrap();
        assert_eq!(6, actual.encoded_len());
        assert_eq!(expect, actual);
    }

    #[test]
    fn equal_string() {
        let utf8 = UTF8String::from("MQTT");
        assert_eq!(String::from("MQTT"), utf8);
    }

    #[test]
    fn length() {
        assert_eq!(12, UTF8String::from("SOMESTRING").encoded_len());
        assert_eq!(11, UTF8String::from("DOLLAR€").encoded_len());
    }
}