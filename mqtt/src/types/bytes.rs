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
    
    /// Clones and returns the inner vec of bytes
    pub fn clone_inner(&self) -> Vec<u8> {
        self.inner.clone()
    }
}

impl MqttDataType for BinaryData {
    /// The length of the binary data plus 2 bytes for the full binary representation in an MQTT packet.
    fn encoded_len(&self) -> usize {
        self.inner.len() + 2
    }
}

impl From<BinaryData> for Vec<u8> {
    fn from(src: BinaryData) -> Self {
        let len = src.inner.len();
        let mut result = Vec::with_capacity(len + 2);
        
        let length = len as u16;
        for b in length.to_be_bytes() {
            result.push(b)
        }

        for b in src.inner {
            result.push(b);
        }

        result
    }
}

impl TryFrom<&[u8]> for BinaryData {
    type Error = MqttError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 2 {
            return Err(MqttError::Message("Binary data must be at least two bytes long!".to_string()))
        }

        let (len, val) = value.split_at(2);
        let length = u16::from_be_bytes(len.try_into().unwrap()) as usize;
        if length > val.len() {
            return Err(MqttError::Message(format!("Message too short. Specified [{}] and actual [{}] length mismatch", length, val.len())))
        }

        let mut inner: Vec<u8> = Vec::with_capacity(length);
        inner.extend_from_slice(&val[..length]); // copy_from_slice() maybe?

        // not sure if we really need the length validation from the new() function
        BinaryData::new(inner)
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
    fn decode_length_too_short() {
        let bytes = [0, 6, 129, 90, 3, 240, 7];
        assert!(BinaryData::try_from(&bytes[..]).is_err());
    }

    #[test]
    fn decode_extract_from_longer_slice() {
        let bytes: Vec<u8> = vec![0,29,123,39,115,39,58,39,115,101,110,115,111,114,39,44,39,108,39,58,39,107,105,116,99,104,101,110,39,32,125,0,6,109,121,110,97,109,101];
        let bin = BinaryData::try_from(&bytes[..]).unwrap();
        assert_eq!(31, bin.encoded_len());
    }

}