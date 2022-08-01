use crate::error::MqttError;

use super::MqttDataType;

/// MQTT-3.2.2.2: Connect Reason Codes, a single byte numeric value.
/// Anything above 0x80 is considered an error. 
/// See the spec for details.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReasonCode {
    /// 0x00
    Success = 0x00,
    /// 0x80
    UnspecifiedError = 0x80,
    /// 0x81
    MalformedPacket = 0x81,
    /// 0x82
    ProtocolError = 0x82,
    /// 0x83
    ImplementationSpecificError = 0x83,
    /// 0x84
    UnsupportedProtocolVersion = 0x84,
    /// 0x85
    ClientIdentifierInvalid = 0x85,
    /// 0x86
    BadUserNameOrPassword = 0x86,
    /// 0x87
    NotAuthorized = 0x87,
    /// 0x88
    ServerUnavailable = 0x88,
    /// 0x89
    ServerBusy = 0x89,
    /// 0x8A
    Banned = 0x8A,
    /// 0x8C
    BadAuthenticationMethod = 0x8C,
    /// 0x90
    TopicNameInvalid = 0x90,
    /// 0x95
    PacketTooLarge = 0x95,
    /// 0x97
    QuotaExceeded = 0x97,
    /// 0x99
    PayloadFormatInvalid = 0x99,
    /// 0x9A
    RetainNotSupported = 0x9A,
    /// 0x98
    QoSNotSupported = 0x9B,
    /// 0x9C
    UseAnotherServer = 0x9C,
    /// 0x9D
    ServerMoved = 0x9D,
    /// 0x9F
    ConnectionRateExceeded = 0x9F,
}

impl ReasonCode {

    /// Returns `true` if the reason code has a numeric value of 0x80 or higher.
    pub fn is_err(&self) -> bool {
        let num = *self as u8;
        num <= 128
    }
}

impl MqttDataType for ReasonCode {
    fn encoded_len(&self) -> usize {
        1
    }
}

impl Into<u8> for ReasonCode {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for ReasonCode {
    type Error = MqttError;

    /// Converts numeric values to a reason code enum, or returns an error if the code is undefined
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            128 => Ok(Self::UnspecifiedError),
            129 => Ok(Self::MalformedPacket),
            130 => Ok(Self::ProtocolError),
            131 => Ok(Self::ImplementationSpecificError),
            132 => Ok(Self::UnsupportedProtocolVersion),
            133 => Ok(Self::ClientIdentifierInvalid),
            134 => Ok(Self::BadUserNameOrPassword),
            135 => Ok(Self::NotAuthorized),
            136 => Ok(Self::ServerUnavailable),
            137 => Ok(Self::ServerBusy),
            138 => Ok(Self::Banned),
            140 => Ok(Self::BadAuthenticationMethod),
            144 => Ok(Self::TopicNameInvalid),
            149 => Ok(Self::PacketTooLarge),
            151 => Ok(Self::QuotaExceeded),
            153 => Ok(Self::PayloadFormatInvalid),
            154 => Ok(Self::RetainNotSupported),
            155 => Ok(Self::QoSNotSupported),
            156 => Ok(Self::UseAnotherServer),
            157 => Ok(Self::ServerMoved),
            159 => Ok(Self::ConnectionRateExceeded),
            _=> Err(MqttError::Message(format!("Undefined Reason Code: {}", value))),
        }        
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;

    #[test]
    fn test_reason_code_conversions() {
        assert_eq!(Ok(ReasonCode::Success), ReasonCode::try_from(0));
        assert_eq!(Ok(ReasonCode::ServerBusy), ReasonCode::try_from(137));
        assert_eq!(Ok(ReasonCode::TopicNameInvalid), ReasonCode::try_from(144));
        assert_eq!(Ok(ReasonCode::ProtocolError), ReasonCode::try_from(130));
        assert_eq!(Ok(ReasonCode::Banned), ReasonCode::try_from(138));
        
        let err1 = ReasonCode::try_from(0xFF);
        assert!(err1.is_err());
        assert_eq!(Some(MqttError::Message("Undefined Reason Code: 255".to_string())), err1.err());

        let err2 = ReasonCode::try_from(0xBA);
        assert!(err2.is_err());
        assert_eq!(Some(MqttError::Message("Undefined Reason Code: 186".to_string())), err2.err());
    }
}