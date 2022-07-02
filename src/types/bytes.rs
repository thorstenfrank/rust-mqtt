use crate::error::MqttError;

use super::MqttDataType;

const MAX_LENGTH: usize = u16::MAX as usize;

/// A simple wrapper around a vector of bytes
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryData {
    inner: Vec<u8>,
}

impl BinaryData {

    /// Returns an `MqttError` if the vector exceeds the maximum allowed number of bytes (65535).
    pub fn new(bytes: Vec<u8>) -> Result<Self, MqttError> {
        if bytes.len() > MAX_LENGTH {
            return Err(MqttError::Message("Max length for binary elements is 65535".to_string()));
        }

        Ok(BinaryData { inner: bytes})
    }
}

impl MqttDataType for BinaryData {}

impl Into<Vec<u8>> for BinaryData {
    
    fn into(self) -> Vec<u8> {
        let len = self.inner.len();
        let mut result = Vec::with_capacity(len + 2);
        
        let length = len as u16;
        for b in length.to_be_bytes() {
            result.push(b)
        }

        for b in self.inner {
            result.push(b);
        }

        result
    }
}

impl TryFrom<&[u8]> for BinaryData {
    type Error = MqttError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 2 {
            return Err(MqttError::Message("".to_string()))
        }

        let (len, val) = value.split_at(2);
        let length = u16::from_be_bytes(len.try_into().unwrap());
        if length as usize != val.len() {
            return Err(MqttError::Message(format!("Specified [{}] and actual [{}] length of binary data does not match", length, val.len())))
        }

        let mut inner: Vec<u8> = Vec::with_capacity(val.len());
        inner.extend_from_slice(val); // copy_from_slice() maybe?

        Ok(BinaryData { inner })
    }
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    #[test]
    fn exceeding_max_capacity() {
        assert!(BinaryData::new(vec![1; 65535]).is_ok());
        assert!(BinaryData::new(vec![1; 65536]).is_err());
    }

    #[test]
    fn encode() {
        let bytes = BinaryData::new(vec![0, 1, 2, 3, 4, 5]).unwrap();

        let expected: Vec<u8> = vec![0, 6, 0, 1, 2, 3, 4, 5];
        let actual: Vec<u8> = bytes.into();

        assert_eq!(expected, actual);
    }

    #[test]
    fn decode() {
        let bytes = [0, 5, 129, 90, 3, 240, 7];
        let actual = BinaryData::try_from(&bytes[..]).unwrap();
        assert_eq!(BinaryData { inner: vec![129, 90, 3, 240, 7]}, actual)
    }

    #[test]
    fn decode_too_short() {
        let bytes = [2];
        assert!(BinaryData::try_from(&bytes[..]).is_err());
    }

    #[test]
    fn decode_length_mismatch() {
        let bytes = [0, 4, 129, 90, 3, 240, 7];
        assert!(BinaryData::try_from(&bytes[..]).is_err());
    }

}