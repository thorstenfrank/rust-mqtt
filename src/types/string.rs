use std::fmt::Display;

use crate::error::MqttError;

use super::MqttDataType;

/// MQTT-1.5.4
#[derive(Debug, PartialEq, Hash, Eq)]
pub struct UTF8String {
    value: String,
}

impl MqttDataType for UTF8String {
    fn encoded_len(&self) -> usize {
        self.value.len() + 2
    }
}

impl UTF8String {

    const LENGTH_FIELD_SIZE: usize = 2;

    pub fn new(value: String) -> Self {
        // TODO add validation for UTF-8 compliance
        UTF8String { value }
    }

    pub fn from_str(value: &str) -> Self {
        // TODO add validation for UTF-8 compliance
        UTF8String { value: value.to_string() }
    }

    /// Returns number of bytes of this string including the 2 bytes holding the length.
    #[deprecated = "use MqttDataType::encoded_len() instead"]
    pub fn len(&self) -> u16 {
        self.value.len() as u16 + 2 // adds the u16 length field
    }
}

impl Into<Vec<u8>> for UTF8String {
    fn into(self) -> Vec<u8> {
        let bytes = self.value.as_bytes();
        
        // FIXME add validation for MAX_LENGTH = 65,535 bytes
        let mut result = Vec::new();

        let length = bytes.len() as u16;
        for b in length.to_be_bytes() {
            result.push(b)
        }

        for b in self.value.as_bytes() {
            result.push(*b)
        }

        result
    }
}

// FIXME change this to TryFrom
impl TryFrom<&[u8]> for UTF8String {

    type Error = MqttError;
    
    fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
        let (len_slice, value) = src.split_at(UTF8String::LENGTH_FIELD_SIZE);
        let length:usize = u16::from_be_bytes([len_slice[0], len_slice[1]]).into();
        
        match String::from_utf8(value[..length].to_vec()) {
            Ok(s) => Ok(UTF8String::new(s)),
            Err(e) => Err(MqttError::Message(format!("Error decoding bytes to String: {:?}", e))),
        }
    }
}

impl From<&str> for UTF8String {
    fn from(src: &str) -> Self {
        UTF8String::new(src.to_string())
    }
}

impl PartialEq<UTF8String> for String {
    fn eq(&self, other: &UTF8String) -> bool {
        self.eq(&other.value)
    }
}

impl Display for UTF8String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use std::{vec, str::FromStr};

    use super::*;

    #[test]
    fn encode_utf8() {
        let utf8 = UTF8String::from_str("MQTT");
        let expect: Vec<u8> = vec![0, 4, 77, 81, 84, 84];
        let actual: Vec<u8> = utf8.into();
        assert_eq!(expect, actual);
    }

    #[test]
    fn encode_empty() {
        let utf8 = UTF8String::new(String::new());
        let expect: Vec<u8> = vec![0, 0];
        let actual: Vec<u8> = utf8.into();
        assert_eq!(expect, actual);
    }

    #[test]
    fn decode_utf8() {
        let source: Vec<u8> = vec![0, 4, 77, 81, 84, 84];
        let expect = UTF8String::new(String::from_str("MQTT").unwrap());
        let actual = UTF8String::try_from(source.as_slice()).unwrap();
        assert_eq!(6, actual.encoded_len());
        assert_eq!(expect, actual);
    }

    #[test]
    fn equal_string() {
        let utf8 = UTF8String::from_str("MQTT");
        assert_eq!(String::from("MQTT"), utf8);
    }

    #[test]
    fn length() {
        assert_eq!(12, UTF8String::from_str("SOMESTRING").encoded_len());
        assert_eq!(11, UTF8String::from_str("DOLLAR€").encoded_len());
    }
}