use crate::error::MqttError;

/// Quality of Service levels.
/// See [the spec](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901234).
#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum QoS {
    /// 0
    AtMostOnce = 0,
    /// 1
    AtLeastOnce = 1,
    /// 2
    ExactlyOnce = 2,
}

impl TryFrom<u8> for QoS {
    type Error = MqttError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(QoS::AtMostOnce),
            1 => Ok(QoS::AtLeastOnce),
            2 => Ok(QoS::ExactlyOnce),
            _=> Err(MqttError::MalformedPacket(format!("Illegal value for QoS: {}", value))),
        }
    }
}

impl Into<u8> for QoS {
    fn into(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn qos_try_from_u8() {
        assert_eq!(QoS::AtMostOnce, QoS::try_from(0_u8).unwrap());
        assert_eq!(QoS::AtLeastOnce, QoS::try_from(1_u8).unwrap());
        assert_eq!(QoS::ExactlyOnce, QoS::try_from(2_u8).unwrap());
        assert!(QoS::try_from(3_u8).is_err());
    }
}