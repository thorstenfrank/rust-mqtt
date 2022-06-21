//! Common data types as defined by the MQTT spec.

/// MQTT-1.5.5
#[derive(Debug, PartialEq)]
pub struct VariableByteInteger {
    pub value: u32,
}

/// MQTT-1.5.4
#[derive(Debug, PartialEq)]
pub struct UTF8String {
    value: String,
}

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


impl From<&[u8]> for VariableByteInteger {
    
    /// Attempts to read an unsigned integer (between 7 and 28 bits) value from one to four bytes
    /// according to the MQTT Spec 1.5.5.
    fn from(bytes: &[u8]) -> Self {
        // FIXME add validation (max length = 4)
        let mut value: u32 = 0;
        let mask: u8 = 127;
        let mut multiplier: u32 = 1;

        for byte in bytes {
            let masked: u32 = (byte & mask) as u32;
            value += (masked * multiplier) as u32;
            multiplier *= 128;

            if byte & 128 == 0 {
                println!("{} and 128 == 0, skipping", byte);
                break
            }
        }

        VariableByteInteger { value }
    }
}

impl Into<Vec<u8>> for VariableByteInteger {

    /// Converts an unsigned integer (max 28 bits) into the binary representation according to MQTT Spec 1.5.5.
    fn into(self) -> Vec<u8> {
        // FIXME add validation to make sure the value does not exceed the max (268,435,455)
        let mut res: Vec<u8> = Vec::new();
        let mut val = self.value;

        while val > 0 {
            let mut byte: u8 = (val % 128) as u8;
            val = val / 128;
            if val > 0 {
                byte = byte | 128;
            }
            res.push(byte);
        }
        res
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
    fn test_encode_vbi() {
        do_test_encode_vbi(16, vec![16]);
        do_test_encode_vbi(128, vec![128, 1]);
        do_test_encode_vbi(129, vec![129, 1]);
        do_test_encode_vbi(2097151, vec![0xFF, 0xFF, 0x7F]);
    }

    fn do_test_encode_vbi(value: u32, expect: Vec<u8>) {
        let actual: Vec<u8> = VariableByteInteger{ value }.into();
        assert_eq!(expect, actual, "error trying to encode {}", value);
    }

    #[test]
    fn test_decode_vbi() {
        do_test_decode_vbi(&vec![78], 78);
        do_test_decode_vbi(&vec![129, 1], 129);
        do_test_decode_vbi(&vec![0x80, 0x80, 0x80, 0x01], 2097152);
    }

    fn do_test_decode_vbi(bytes: &[u8], expect: u32) {
        let actual = VariableByteInteger::from(bytes);
        assert_eq!(expect, actual.value, "error trying to decode into {}", expect);
    }

    #[test]
    fn test_encode_utf8() {
        let utf8 = UTF8String::from_str("MQTT");
        let expect: Vec<u8> = vec![0, 4, 77, 81, 84, 84];
        let actual: Vec<u8> = utf8.into();
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_decode_utf8() {
        let source: Vec<u8> = vec![0, 4, 77, 81, 84, 84];
        let expect = UTF8String::new(String::from_str("MQTT").unwrap());
        let actual = UTF8String::from(source.as_slice());
        assert_eq!(expect, actual);
    }

}