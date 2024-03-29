//! Primarily implements the `Variable Byte Integer` type, but also adds trait impls for basic rust types.

use crate::error::MqttError;

use super::MqttDataType;

/// Unsigned, Big-Endian integer value, represented from 8 to 24 bits.
/// See [MQTT-1.5.5](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901011).
/// 
/// Internally uses a `u32`, but encodes to 1-4 bytes depending on the value.
#[derive(Debug, PartialEq)]
pub struct VariableByteInteger {
    pub value: u32,
}

impl MqttDataType for VariableByteInteger {
    fn encoded_len(&self) -> usize {
        match self.value {
            x if x <= 127 => 1,
            x if x <= 16383 => 2,
            x if x <= 2097151 => 3,
            _=> 4,
        }
    }
}

impl TryFrom<&[u8]> for VariableByteInteger {
    type Error = MqttError;

    /// Attempts to read an unsigned integer (between 7 and 28 bits) value from one to four bytes
    /// according to the MQTT Spec 1.5.5.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        // FIXME add validation (max length = 4)
        let mut value: u32 = 0;
        let mask: u8 = 127;
        let mut multiplier: u32 = 1;

        for byte in bytes {
            let masked: u32 = (byte & mask) as u32;
            value += (masked * multiplier) as u32;
            multiplier *= 128;

            // stop at the first byte where the LSB is no set
            if byte & 128 == 0 {
                break
            }
        }

        Ok(VariableByteInteger { value })
    }

}

impl From<u32> for VariableByteInteger {
    fn from(value: u32) -> Self {
        VariableByteInteger { value }
    }
}

impl From<VariableByteInteger> for Vec<u8> {

    /// Converts an unsigned integer (max 28 bits) into the binary representation according to MQTT Spec 1.5.5.
    fn from(src: VariableByteInteger) -> Self {
        // FIXME add validation to make sure the value does not exceed the max (268,435,455)
        let mut res: Vec<u8> = Vec::new();
        let mut val = src.value;

        if val == 0 {
            return vec![0]
        }

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

/* 
  Impls for primitive datatypes.
  These map to MQTT spec types `Byte`, `Two Byte Integer` and `Four Byte Integer`
 */
impl MqttDataType for u8 {
    fn encoded_len(&self) -> usize {
        // std::mem::size_of::<u8>()
        1
    }
}
impl MqttDataType for u16 {
    fn encoded_len(&self) -> usize {
        2
    }
}
impl MqttDataType for u32 {
    fn encoded_len(&self) -> usize {
        4
    }
}

#[cfg(test)]
mod tests {
    use std::{vec};

    use super::*;

    #[test]
    fn encode_vbi() {
        do_test_encode_vbi(0, vec![0]);
        do_test_encode_vbi(16, vec![16]);
        do_test_encode_vbi(128, vec![128, 1]);
        do_test_encode_vbi(129, vec![129, 1]);
        do_test_encode_vbi(2097151, vec![0xFF, 0xFF, 0x7F]);
    }

    #[test]
    fn decode_vbi() {
        do_test_decode_vbi(&vec![78], 78);
        do_test_decode_vbi(&vec![129, 1], 129);
        do_test_decode_vbi(&vec![0x80, 0x80, 0x80, 0x01], 2097152);
        do_test_decode_vbi(&vec![0], 0);
    }

    #[test]
    fn vbi_size() {
        assert_eq!(1, VariableByteInteger{value: 84}.encoded_len());
        assert_eq!(1, VariableByteInteger{value: 127}.encoded_len());
        assert_eq!(2, VariableByteInteger{value: 128}.encoded_len());
        assert_eq!(2, VariableByteInteger{value: 8342}.encoded_len());
        assert_eq!(2, VariableByteInteger{value: 16383}.encoded_len());
        assert_eq!(3, VariableByteInteger{value: 16384}.encoded_len());
        assert_eq!(3, VariableByteInteger{value: 2097151}.encoded_len());
        assert_eq!(4, VariableByteInteger{value: 2097152}.encoded_len());
        assert_eq!(4, VariableByteInteger{value: 268435455}.encoded_len());

        // this is dumb as it exceeds the MQTT spec max value, but hey
        assert_eq!(4, VariableByteInteger{value: u32::MAX}.encoded_len());
    }

    fn do_test_encode_vbi(value: u32, expect: Vec<u8>) {
        let actual: Vec<u8> = VariableByteInteger{ value }.into();
        assert_eq!(expect, actual, "error trying to encode {}", value);
    }
    
    fn do_test_decode_vbi(bytes: &[u8], expect: u32) {
        let actual = VariableByteInteger::try_from(bytes).unwrap();
        assert_eq!(expect, actual.value, "error trying to decode into {}", expect);
    }

}