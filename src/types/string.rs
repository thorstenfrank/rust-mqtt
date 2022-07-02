use super::MqttDataType;

/// MQTT-1.5.4
#[derive(Debug, PartialEq)]
pub struct UTF8String {
    value: String,
}

impl MqttDataType for UTF8String {}

impl UTF8String {

    pub fn new(value: String) -> Self {
        // TODO add validation for UTF-8 compliance
        UTF8String { value }
    }

    pub fn from_str(value: &str) -> Self {
        // TODO add validation for UTF-8 compliance
        UTF8String { value: value.to_string() }
    }

    pub fn len(&self) -> u16 {
        self.value.len() as u16
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
impl From<&[u8]> for UTF8String {
    fn from(src: &[u8]) -> Self {
        // FIXME yeah, yeah, yeah, I know, we need to actually read the length of the string
        // then read it, THEN make sure it's actually UTF-8 and blablabla
        let mut the_string: Vec<u8> = Vec::new();
        for b in &src[2..] {
            the_string.push(*b)
        }
        UTF8String::new(String::from_utf8(the_string).unwrap())
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
    fn decode_utf8() {
        let source: Vec<u8> = vec![0, 4, 77, 81, 84, 84];
        let expect = UTF8String::new(String::from_str("MQTT").unwrap());
        let actual = UTF8String::from(source.as_slice());
        assert_eq!(expect, actual);
    }
}